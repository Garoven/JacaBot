use std::process::{Command, Stdio};

pub fn playlist(uri: &str) -> Option<Vec<String>> {
    if let Ok(output) = Command::new("youtube-dl")
        .args([
            "-f",
            "webm[abr>0]/bestaudio/best",
            "--yes-playlist",
            "--flat-playlist",
            "--dump-single-json",
            "--clean-info-json",
            "--skip-download",
            uri,
        ])
        .stdin(Stdio::null())
        .output()
    {
        let json: serde_json::Value = match serde_json::from_reader(&output.stdout[..]) {
            Ok(ok) => ok,
            Err(_) => return None,
        };
        let map = match json.as_object() {
            Some(map) => map,
            None => return None,
        };
        let entries = match map.get("entries").and_then(serde_json::Value::as_array) {
            Some(data) => data,
            None => return None,
        };
        let mut vec: Vec<String> = Vec::new();
        for (index, _) in entries.iter().enumerate() {
            let single_entry = match entries.get(index).and_then(serde_json::Value::as_object) {
                Some(entry) => entry,
                None => continue,
            };
            let entry_url = match single_entry.get("url").and_then(serde_json::Value::as_str) {
                Some(url) => url,
                None => continue,
            };
            vec.push(entry_url.to_string());
        }
        Some(vec)
    } else {
        None
    }
}
