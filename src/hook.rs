use std::cell::RefCell;
use std::{mem, ptr};

use winapi::shared::{minwindef, ntdef, windef};
use winapi::um::{errhandlingapi, processthreadsapi, winbase, winuser};

use config::Config;
use input::*;

const MSG_EXIT: minwindef::UINT = winuser::WM_USER + 1;

// TODO maybe move this to an AtomicPointer
thread_local!{
    static HOOK: RefCell<Option<Hook>> = RefCell::new(None);
}

pub struct Hook {
    id: windef::HHOOK,
    tid: minwindef::DWORD,
    config: Config,
    paused: bool,
    fg: Option<windef::HWND>,
    cb: Box<FnMut(Event)>,
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe { winuser::PostThreadMessageW(self.tid, MSG_EXIT, 0, 0) };
    }
}

impl Hook {
    pub fn run(config: Config, f: impl FnMut(Event) + 'static) {
        let id = unsafe {
            winuser::SetWindowsHookExW(winuser::WH_KEYBOARD_LL, Some(callback), ptr::null_mut(), 0)
        };

        if id.is_null() {
            error!("error: {}", get_last_error());
            return;
        }

        let tid = unsafe { processthreadsapi::GetCurrentThreadId() };
        HOOK.with(|hook| {
            *hook.borrow_mut() = Some(Hook {
                id,
                tid,
                config,
                paused: true,
                fg: None,
                cb: Box::new(f),
            });
        });

        // dropping the Hook will send the MSG_EXIT message, which'll exit this loop
        let mut msg = unsafe { mem::uninitialized() };
        loop {
            let ret = unsafe { winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
            match (ret, msg.message) {
                (ret, _) if ret < 0 => {
                    error!("error: {}", get_last_error());
                    break;
                }
                (_, MSG_EXIT) => {
                    debug!("got exit message");
                    break;
                }
                _ => {}
            }
        }

        unsafe { winuser::UnhookWindowsHookEx(id) };
    }
}

unsafe extern "system" fn callback(
    code: i32,
    wp: minwindef::WPARAM,
    lp: minwindef::LPARAM,
) -> minwindef::LRESULT {
    HOOK.with(|hook| {
        let mut hook = hook.borrow_mut();
        let hook = hook.as_mut().expect("to borrow hook");

        match wp as u32 {
            winuser::WM_KEYUP | winuser::WM_KEYDOWN => {
                let kb: winuser::KBDLLHOOKSTRUCT = *(lp as *const winuser::KBDLLHOOKSTRUCT);

                let key = match KEY_TABLE[kb.vkCode as usize] {
                    Some(key) => key,
                    None => match Key::try_make(
                        kb.scanCode as usize,
                        false,
                        u32::from(winuser::KF_ALTDOWN >> 8) & kb.flags == 1,
                    ) {
                        Some(key) => key,
                        None => return winuser::CallNextHookEx(hook.id, code, wp, lp),
                    },
                };

                let state = if (wp as u32) == winuser::WM_KEYUP {
                    KeyState::Released
                } else {
                    KeyState::Pressed
                };

                let hwnd = winuser::GetForegroundWindow();

                if state == KeyState::Pressed {
                    if key == hook.config.exit_key {
                        info!("quitting");
                        winuser::PostThreadMessageW(hook.tid, MSG_EXIT, 0, 0);
                        // skip the dispatch
                        return winuser::CallNextHookEx(hook.id, code, wp, lp);
                    }
                    if key == hook.config.watch_key && !hook.paused {
                        hook.fg = Some(hwnd);
                        info!("watching {}", get_window_title(hwnd));
                        // skip the dispatch
                        return winuser::CallNextHookEx(hook.id, code, wp, lp);
                    }
                    if key == hook.config.pause_key {
                        hook.paused = !hook.paused;
                        info!("setting pause to: {}", hook.paused);
                        // skip the dispatch
                        return winuser::CallNextHookEx(hook.id, code, wp, lp);
                    }
                }

                if !hook.paused && Some(hwnd) == hook.fg {
                    let ev = Event::new(key, state);
                    (&mut hook.cb)(ev)
                }
            }

            _ => {}
        }

        winuser::CallNextHookEx(hook.id, code, wp, lp)
    })
}

fn get_last_error() -> String {
    let err = unsafe { errhandlingapi::GetLastError() };

    let mut msg = vec![0u16; 127]; // fixed length strings in the windows api
    unsafe {
        winbase::FormatMessageW(
            winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null_mut(),
            err,
            u32::from(ntdef::LANG_SYSTEM_DEFAULT),
            msg.as_mut_ptr(),
            msg.len() as u32,
            ptr::null_mut(),
        );
    }
    String::from_utf16_lossy(&msg)
}

fn get_window_title(hwnd: windef::HWND) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let len = unsafe { winuser::GetWindowTextLengthW(hwnd) + 1 };
    let mut buf = vec![0u16; len as usize];
    let len = unsafe { winuser::GetWindowTextW(hwnd, buf.as_mut_ptr(), len) };
    buf.truncate(len as usize);

    OsString::from_wide(&buf).to_string_lossy().to_string()
}
