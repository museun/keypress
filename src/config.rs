use input::Key;
use std::{fs, io::Write};
use toml;

pub const CONFIG_FILE: &str = "keypressed.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub exit_key: Key,
    pub watch_key: Key,
    pub pause_key: Key,
    pub host: String,
}

pub enum Error {
    Read,
    Write,
    Serialize,
    Deserialize,
}

pub fn load_or_create_config() -> Config {
    match Config::load() {
        Ok(config) => config,
        Err(Error::Read) => {
            let config = Config {
                exit_key: Key::VK_F10,
                watch_key: Key::VK_F9,
                pause_key: Key::VK_PAUSE,
                host: "localhost:51005".into(),
            };

            if config.save().is_err() {
                error!("cannot create default config");
                ::std::process::exit(1);
            }

            warn!("created a default config in {}", CONFIG_FILE);
            warn!("edit it then rerun the program");
            ::std::process::exit(1);
        }
        Err(_) => {
            error!("cannot load the config file",);
            ::std::process::exit(1);
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let data = fs::read_to_string(CONFIG_FILE).map_err(|e| {
            error!("cannot read file: {}", e);
            Error::Read
        })?;
        toml::from_str(&data).map_err(|e| {
            error!("Cannot deserialize toml: {}", e);
            Error::Deserialize
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        let toml = toml::to_string(&self).map_err(|e| {
            error!("cannot serialize toml: {}", e);
            Error::Serialize
        })?;

        let mut f = fs::File::create(CONFIG_FILE).map_err(|e| {
            error!("cannot create file: {}", e);
            Error::Write
        })?;

        f.write_all(toml.as_bytes()).map_err(|e| {
            error!("cannot create file: {}", e);
            Error::Write
        })
    }
}
