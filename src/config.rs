use std::{
    collections::HashMap,
    fs::read_to_string,
    process::exit
};
use crate::{log, Log};
use once_cell::sync::Lazy;

static CONFIG: Lazy<String> = Lazy::new(|| {
    match read_to_string("./config.json") {
        Ok(data) => {
            log("Config file found", Log::Info());
            data
        },
        Err(_) => {
            log("Config file not found", Log::Error());
            exit(2)
        }
    }
});

pub static DISCORD_CONFIG: Lazy<serde_json::Value> = Lazy::new(|| {
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&CONFIG).unwrap();
    data.get("discord").unwrap().to_owned()
});

pub static SPOTIFY_CONFIG: Lazy<serde_json::Value> = Lazy::new(|| {
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&CONFIG).unwrap();
    data.get("spotify").unwrap().to_owned()
});
