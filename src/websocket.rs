use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use input::{Event, Key};

use crossbeam_channel as channel;
use serde_json;
use tungstenite as ws;

type TX<T> = channel::Sender<T>;
type RX<T> = channel::Receiver<T>;

#[derive(Serialize)]
pub struct Message {
    #[serde(with = "string")]
    pub key: Key,
    pub l_shift: bool,
    pub r_shift: bool,
    pub l_control: bool,
    pub r_control: bool,
    pub l_alt: bool,
    pub r_alt: bool,
}

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl From<Event> for Message {
    fn from(ev: Event) -> Self {
        Message {
            key: ev.key,
            l_shift: ev.modifier.left_shift(),
            r_shift: ev.modifier.right_shift(),
            l_control: ev.modifier.left_ctrl(),
            r_control: ev.modifier.right_ctrl(),
            l_alt: ev.modifier.left_alt(),
            r_alt: ev.modifier.right_alt(),
        }
    }
}

pub fn run_in_thread(addr: &str) -> TX<Message> {
    let (tx, rx) = channel::bounded(32);

    let listener = TcpListener::bind(addr).expect("must listen");
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let rx = rx.clone();
                handle_socket(stream, &rx);
            }
        }
    });

    tx
}

fn handle_socket(socket: TcpStream, rx: &RX<Message>) {
    let mut socket = match ws::accept(socket) {
        Ok(val) => val,
        Err(err) => {
            error!("cannot accept client: {}", err);
            return;
        }
    };

    loop {
        select!{
            recv(rx, msg) => {
                if let Some(msg) = msg {
                    let msg = serde_json::to_string(&msg).unwrap(); // trust ourselves
                    if let Err(err) = socket.write_message(ws::Message::Text(msg)) {
                        if let Err(err) = socket.get_mut().shutdown(Shutdown::Both) {
                            debug!("couldn't properly shutdown socket: {}", err);
                        }
                        debug!("couldn't write to client: {}", err);
                        return;
                    }
                }
            }
            // maybe send a ping here
        }
    }
}
