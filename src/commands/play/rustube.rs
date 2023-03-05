use std::{time::Duration, process::Stdio};

use reqwest::Url;
pub use rustube::get_playlist;
use serenity::async_trait;
use songbird::input::{Restartable, restartable::Restart, Metadata, Codec, Container, error::Result, Input, children_to_reader};
use std::process::Command;

struct RustubeRestarter<P> 
where
    P: AsRef<str> + Send + Sync,
{
    uri: P
}

#[async_trait]
impl<P> Restart for RustubeRestarter<P> 
where
    P: AsRef<str> + Send + Sync,
{
    async fn lazy_init(&mut self) -> Result<(Option<Metadata>, Codec, Container)> {
        let m = rustube_metadata(self.uri.as_ref()).await?;
        Ok((Some(m), Codec::FloatPcm, Container::Raw))
    }
    
    async fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        if let Some(time) = time {
            let ts = format!("{:.3}", time.as_secs_f64());

            _rustyt(self.uri.as_ref(), &["-ss", &ts]).await
        } else {
            rustyt(self.uri.as_ref()).await
        }
    }
}

pub async fn rustube<P: AsRef<str> + Send + Clone + Sync + 'static>(uri: P, lazy: bool) -> Result<Restartable> {
    Restartable::new(RustubeRestarter { uri }, lazy).await
}

async fn rustube_metadata(uri: &str) -> Result<Metadata> {
    let url = Url::parse(uri).unwrap();
    let video = match rustube::Video::from_url(&url).await {
        Ok(v) => v,
        Err(_) => return Err(songbird::input::error::Error::Metadata),
    };
    let video_details = video.video_details().clone();
    Ok(Metadata {
        track: None,
        artist: Some(video_details.author.to_owned()),
        date: None,
        channels: Some(2),
        channel: Some(video_details.channel_id.to_owned()),
        start_time: None,
        duration: Some(Duration::from_secs(video_details.length_seconds)),
        sample_rate: None,
        source_url: Some(uri.to_string()),
        title: Some(video_details.title.to_owned()),
        thumbnail: Some(video_details.thumbnails.last().unwrap().url.to_owned())
    })
}


pub async fn rustyt(uri: impl AsRef<str>) -> Result<Input> {
    _rustyt(uri.as_ref(), &[]).await
}

async fn _rustyt(uri: &str, pre_args: &[&str]) -> Result<Input> {
    let url = Url::parse(uri).unwrap();
    let video = match rustube::Video::from_url(&url).await {
        Ok(v) => v,
        Err(_) => return Err(songbird::input::error::Error::Metadata)
    };
    let video_url = video.best_audio().unwrap().signature_cipher.url.as_str();
    let video_details = video.video_details().clone();
    let metadata = Metadata {
        track: None,
        artist: Some(video_details.author.to_owned()),
        date: None,
        channels: Some(2),
        channel: Some(video_details.channel_id.to_owned()),
        start_time: None,
        duration: Some(Duration::from_secs(video_details.length_seconds)),
        sample_rate: None,
        source_url: Some(uri.to_string()),
        title: Some(video_details.title.to_owned()),
        thumbnail: Some(video_details.thumbnails.last().unwrap().url.to_owned())
    };
    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];
    let mut curl = Command::new("curl")
        .arg(video_url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(curl.stdout.take().unwrap())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    Ok(Input::new(
            true,
            children_to_reader::<f32>(vec![curl, ffmpeg]),
            Codec::FloatPcm,
            Container::Raw,
            Some(metadata)
    ))
}

struct RustubeSearchRestarter<P> 
where
    P: AsRef<str> + Send + Sync,
{
    uri: P
}

#[async_trait]
impl<P> Restart for RustubeSearchRestarter<P> 
where
    P: AsRef<str> + Send + Sync,
{
    async fn lazy_init(&mut self) -> Result<(Option<Metadata>, Codec, Container)> {
        let m = rustube_search_metadata(self.uri.as_ref()).await?;
        Ok((Some(m), Codec::FloatPcm, Container::Raw))
    }
    
    async fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        if let Some(time) = time {
            let ts = format!("{:.3}", time.as_secs_f64());

            _rustyt_search(self.uri.as_ref(), &["-ss", &ts]).await
        } else {
            rustyt_search(self.uri.as_ref()).await
        }
    }
}

pub async fn rustube_search<P: AsRef<str> + Send + Clone + Sync + 'static>(uri: P, lazy: bool) -> Result<Restartable> {
    Restartable::new(RustubeSearchRestarter { uri }, lazy).await
}

async fn rustube_search_metadata(uri: &str) -> Result<Metadata> {
    let video = match rustube::get_by_name(uri).await {
        Ok(v) => v,
        Err(_) => return Err(songbird::input::error::Error::Metadata),
    };
    let video_details = video.video_details().clone();
    Ok(Metadata {
        track: None,
        artist: Some(video_details.author.to_owned()),
        date: None,
        channels: Some(2),
        channel: Some(video_details.channel_id.to_owned()),
        start_time: None,
        duration: Some(Duration::from_secs(video_details.length_seconds)),
        sample_rate: None,
        source_url: Some(uri.to_string()),
        title: Some(video_details.title.to_owned()),
        thumbnail: Some(video_details.thumbnails.last().unwrap().url.to_owned())
    })
}


pub async fn rustyt_search(uri: impl AsRef<str>) -> Result<Input> {
    _rustyt(uri.as_ref(), &[]).await
}

async fn _rustyt_search(uri: &str, pre_args: &[&str]) -> Result<Input> {
    let video = match rustube::get_by_name(uri).await {
        Ok(v) => v,
        Err(_) => return Err(songbird::input::error::Error::Metadata)
    };
    let video_url = video.best_audio().unwrap().signature_cipher.url.as_str();
    let video_details = video.video_details().clone();
    let metadata = Metadata {
        track: None,
        artist: Some(video_details.author.to_owned()),
        date: None,
        channels: Some(2),
        channel: Some(video_details.channel_id.to_owned()),
        start_time: None,
        duration: Some(Duration::from_secs(video_details.length_seconds)),
        sample_rate: None,
        source_url: Some(uri.to_string()),
        title: Some(video_details.title.to_owned()),
        thumbnail: Some(video_details.thumbnails.last().unwrap().url.to_owned())
    };
    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];
    let mut curl = Command::new("curl")
        .arg(video_url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(curl.stdout.take().unwrap())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    Ok(Input::new(
            true,
            children_to_reader::<f32>(vec![curl, ffmpeg]),
            Codec::FloatPcm,
            Container::Raw,
            Some(metadata)
    ))
}
