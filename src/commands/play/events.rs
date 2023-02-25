use std::{collections::HashMap, sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::CreateEmbed,
    http::Http,
    model::prelude::{ChannelId, Message},
};

use songbird::{tracks::PlayMode, Event, EventContext, EventHandler};

pub struct SongStart {
    channel_id: ChannelId,
    http: Arc<Http>,
}

impl SongStart {
    pub fn new(chan_id: ChannelId, ctx_http: Arc<Http>) -> Self {
        SongStart {
            channel_id: chan_id,
            http: ctx_http,
        }
    }
}

#[async_trait]
impl EventHandler for SongStart {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_, track)]) = ctx {
            let metadata = track.metadata().clone();

            let title = format!(
                "`{}` by `{}`",
                metadata.title.unwrap(),
                match metadata.artist {
                    Some(name) => name,
                    None => metadata.channel.unwrap(),
                }
            );
            let duration = metadata.duration.unwrap_or(Duration::from_secs(0)).as_secs();
            let minutes_dur = duration / 60;
            let mut seconds_dur = (duration - minutes_dur * 60).to_string();
            if seconds_dur.len() == 1 {
                seconds_dur = "0".to_string() + &seconds_dur;
            }

            let message = self
                .channel_id
                .send_message(&self.http, |response| {
                    response.add_embed(|embed| {
                        embed
                            .title(title)
                            .thumbnail(metadata.thumbnail.unwrap())
                            .description(format!(
                                    "⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪ - `0:00/{}:{}`"
                                    , minutes_dur,
                                    seconds_dur)
                                )
                            .colour(16711937)
                    })
                })
                .await
                .unwrap();

            track
                .add_event(
                    Event::Periodic(Duration::from_secs(5), None),
                    Nowplaying {
                        channel_id: self.channel_id,
                        msg: message.clone(),
                        http: self.http.clone(),
                    },
                )
                .unwrap();

            track
                .add_event(
                    Event::Track(songbird::TrackEvent::End),
                    SongEnd {
                        msg: message,
                        http: self.http.clone(),
                    },
                )
                .unwrap();

            Some(Event::Cancel)
        } else {
            None
        }
    }
}

struct SongEnd {
    msg: Message,
    http: Arc<Http>,
}

#[async_trait]
impl EventHandler for SongEnd {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_, _)]) = ctx {
            self.msg.delete(&self.http).await.unwrap();
            Some(Event::Cancel)
        } else {
            None
        }
    }
}

struct Nowplaying {
    channel_id: ChannelId,
    msg: Message,
    http: Arc<Http>,
}

#[async_trait]
impl EventHandler for Nowplaying {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(state, track)]) = ctx {
            if state.playing == PlayMode::Play {
                let metadata = track.metadata().clone();

                let title = format!(
                    "`{}` by `{}`",
                    metadata.title.unwrap(),
                    match metadata.artist {
                        Some(name) => name,
                        None => metadata.channel.unwrap(),
                    }
                );
                let duration = metadata.duration.unwrap_or(Duration::from_secs(0)).as_secs();
                let minutes_dur = duration / 60;
                let mut seconds_dur = (duration - minutes_dur * 60).to_string();
                if seconds_dur.len() == 1 {
                    seconds_dur = "0".to_string() + &seconds_dur;
                }

                let minutes = state.position.as_secs() / 60;
                let mut seconds = (state.position.as_secs() - minutes * 60).to_string();

                if seconds.len() == 1 {
                    seconds = "0".to_string() + &seconds;
                }

                let mut embed = CreateEmbed(HashMap::new());
                embed
                    .title(title)
                    .thumbnail(metadata.thumbnail.unwrap())
                    .description(format!(
                        "{} - `{}:{}/{}:{}`",
                        time_bar(state.position, metadata.duration.unwrap()),
                        minutes,
                        seconds,
                        minutes_dur,
                        seconds_dur
                    ))
                    .colour(16711937);

                self.channel_id
                    .edit_message(&self.http, self.msg.id, |response| {
                        response.set_embed(embed)
                    })
                    .await
                    .unwrap();
                None
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn time_bar(now: Duration, total: Duration) -> String {
    let proc = (now.as_secs_f64()) / (total.as_secs_f64()) * 100_f64;
    if proc <= 10.0 {
        "⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 20.0 {
        "🔴⚪⚪⚪⚪⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 30.0 {
        "🔴🔴⚪⚪⚪⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 40.0 {
        "🔴🔴🔴⚪⚪⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 50.0 {
        "🔴🔴🔴🔴⚪⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 60.0 {
        "🔴🔴🔴🔴🔴⚪⚪⚪⚪⚪".to_string()
    } else if proc <= 70.0 {
        "🔴🔴🔴🔴🔴🔴⚪⚪⚪⚪".to_string()
    } else if proc <= 80.0 {
        "🔴🔴🔴🔴🔴🔴🔴⚪⚪⚪".to_string()
    } else if proc <= 90.0 {
        "🔴🔴🔴🔴🔴🔴🔴🔴⚪⚪".to_string()
    } else if proc <= 97.0 {
        "🔴🔴🔴🔴🔴🔴🔴🔴🔴⚪".to_string()
    } else {
        "🔴🔴🔴🔴🔴🔴🔴🔴🔴🔴".to_string()
    }
}
