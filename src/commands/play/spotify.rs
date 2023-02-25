use crate::config;

use reqwest::{header::HeaderValue, Method, Url};
use rspotify::{ClientCredsSpotify, Credentials};

use serde_json::Value;

async fn auth() -> String {
    let client_id = config::SPOTIFY_CONFIG.get("client_id").unwrap().as_str().unwrap();
    let client_secret = config::SPOTIFY_CONFIG.get("client_secret").unwrap().as_str().unwrap();
    let credentials = Credentials::new(client_id, client_secret);

    let request = ClientCredsSpotify::new(credentials);
    request.request_token().await.unwrap();
    request
        .token
        .clone()
        .lock()
        .await
        .unwrap()
        .clone()
        .unwrap()
        .access_token
}

pub async fn playlist(uri: &str) -> Option<Vec<String>> {
    let id = uri.split('/').last().unwrap();
    let url = format!("https://api.spotify.com/v1/playlists/{id}?market=PL&fields=tracks.items(track.name%2C%20track.artists.name)");

    let client = reqwest::Client::new();
    let mut reqwest = reqwest::Request::new(Method::GET, Url::parse(&url).unwrap());
    reqwest
        .headers_mut()
        .insert("Accept", HeaderValue::from_static("application/json"));
    reqwest
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));
    reqwest.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", auth().await)).unwrap(),
    );

    let result = client.execute(reqwest).await.unwrap();
    let json: Value = serde_json::from_str(&result.text().await.unwrap()).unwrap();
    let map = match json.as_object().unwrap().get("tracks") {
        Some(val) => val
            .as_object()
            .unwrap()
            .get("items")
            .unwrap()
            .as_array()
            .unwrap(),
        None => return None,
    };

    let mut vec: Vec<String> = Vec::new();
    for obj in map {
        let track = obj.get("track").unwrap();
        let title = track.get("name").unwrap().as_str().unwrap();
        let artists = track.get("artists").unwrap().as_array().unwrap();
        let artist = artists[0].get("name").unwrap().as_str().unwrap();
        vec.push(title.to_string() + " - " + artist);
    }
    Some(vec)
}
