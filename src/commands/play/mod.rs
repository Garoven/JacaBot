use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use songbird::{create_player, input::Restartable, Event};

use super::send_msg;
use crate::commands::edit_msg;
use std::time::Duration;

mod events;
mod spotify;
mod youtube;

use events::SongStart;

pub async fn run(interaction: &ApplicationCommandInteraction, ctx: &Context) {
    let cache = &ctx.cache;

    let uri = interaction.data.options[0]
        .value
        .clone()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id.expect("Not in channel");
    let guild = cache.guild(guild_id).unwrap();

    let voice_channel = guild
        .voice_states
        .get(&user_id)
        .and_then(|voice_state| voice_state.channel_id);

    let menager = songbird::get(ctx)
        .await
        .expect("Failed to get manager")
        .clone();

    if let Some(voice_channel) = voice_channel {
        let handler_lock = menager.get_or_insert(guild_id);
        send_msg(ctx, interaction, "Processing please wait").await;
        if uri.contains("https://") {
            if uri.contains("playlist") {
                let playlist = match uri.contains("spotify") {
                    true => spotify::playlist(&uri).await,
                    false => youtube::playlist(&uri),
                };
                if let Some(vec) = playlist {
                    edit_msg(ctx, interaction, &format!("Adding {} songs", vec.len())).await;
                    for song in vec {
                        let source = match song.contains("https://") {
                            true => match Restartable::ytdl(song, true).await {
                                Ok(src) => src,
                                Err(_) => continue,
                            },
                            false => match Restartable::ytdl_search(song, true).await {
                                Ok(src) => src,
                                Err(_) => continue,
                            },
                        };

                        let (track, track_handle) = create_player(source.into());
                        track_handle
                            .add_event(
                                Event::Periodic(Duration::from_secs(0), None),
                                SongStart::new(interaction.channel_id, ctx.http.clone()),
                            )
                            .unwrap();

                        let mut handler = handler_lock.lock().await;
                        handler.join(voice_channel).await.unwrap();
                        handler.enqueue(track);
                    }
                } else {
                    send_msg(ctx, interaction, "Invalid url").await
                }
            } else if let Ok(source) = Restartable::ytdl(uri, true).await {
                let (track, track_handle) = create_player(source.into());
                track_handle
                    .add_event(
                        Event::Periodic(Duration::from_secs(0), None),
                        SongStart::new(interaction.channel_id, ctx.http.clone()),
                    )
                    .unwrap();

                let metadata = track_handle.metadata().clone();
                let content = format!(
                    "Added `{}` by `{}`",
                    metadata.title.unwrap(),
                    match metadata.artist {
                        Some(name) => name,
                        None => metadata.channel.unwrap(),
                    }
                );

                let mut handler = handler_lock.lock().await;
                handler.join(voice_channel).await.unwrap();
                handler.enqueue(track);

                edit_msg(ctx, interaction, &content).await
            } else {
                send_msg(ctx, interaction, "Nothing found").await
            }
        } else if let Ok(source) = Restartable::ytdl_search(uri, true).await {
            let (track, track_handle) = create_player(source.into());
            track_handle
                .add_event(
                    Event::Periodic(Duration::from_secs(0), None),
                    SongStart::new(interaction.channel_id, ctx.http.clone()),
                )
                .unwrap();

            let metadata = track_handle.metadata().clone();
            let content = format!(
                "Added `{}` by `{}`",
                metadata.title.unwrap(),
                match metadata.artist {
                    Some(name) => name,
                    None => metadata.channel.unwrap(),
                }
            );

            let mut handler = handler_lock.lock().await;
            handler.join(voice_channel).await.unwrap();
            handler.enqueue(track);

            edit_msg(ctx, interaction, &content).await
        } else {
            send_msg(ctx, interaction, "Nothing found").await
        }
    } else {
        send_msg(ctx, interaction, "Not in voice channel").await
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("play")
        .description("play song from yt")
        .create_option(|option| {
            option
                .name("query")
                .description("Insert video/playlist name or url")
                .kind(serenity::model::prelude::command::CommandOptionType::String)
                .required(true)
        })
}
