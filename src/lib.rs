#![allow(dead_code)]
extern crate winapi;

#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

#[macro_use]
extern crate crossbeam_channel;
extern crate tungstenite;

mod config;
pub use config::*;

mod hook;
pub use hook::Hook;

mod input;
pub use input::*;

pub mod websocket;

#[cfg(feature = "generate")]
mod generate;
