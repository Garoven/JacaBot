use std::{
    collections::HashMap,
    fs::read_to_string,
    process::exit, sync::LazyLock
};
use log::error;

static CONFIG: LazyLock<String> = LazyLock::new(|| {
    match read_to_string("./config.json") {
        Ok(data) => data,
        Err(_) => {
            error!("Cannot load config file. Does it exits?");
            exit(1)
        }
    }
});

pub static DISCORD_CONFIG: LazyLock<serde_json::Value> = LazyLock::new(|| {
    let data: HashMap<String, serde_json::Value> = match serde_json::from_str(&CONFIG) {
        Ok(json) => json,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    match data.get("discord") {
        Some(d) => d.to_owned(),
        None => {
            error!("Cannot parse config file. Invalid config?");
            exit(1)
        }
    }
});

pub static SPOTIFY_CONFIG: LazyLock<serde_json::Value> = LazyLock::new(|| {
    let data: HashMap<String, serde_json::Value> = match serde_json::from_str(&CONFIG) {
        Ok(json) => json,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    match data.get("spotify") {
        Some(d) => d.to_owned(),
        None => {
            error!("Cannot parse config file. Invalid config?");
            exit(1)
        }
    }
});
