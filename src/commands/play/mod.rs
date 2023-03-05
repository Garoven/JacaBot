use serenity::{
    builder::CreateApplicationCommand,
    model::{prelude::interaction::application_command::ApplicationCommandInteraction, user::User},
    prelude::Context,
};

use log::trace;
use songbird::{create_player, input::Metadata, Event};

use super::send_msg;
use crate::commands::edit_msg;
use std::time::Duration;

mod rustube;
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

    let user = &interaction.user;
    let user_id = user.id;
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
                    false => Some(rustube::get_playlist(&uri).await.links),
                };
                if let Some(vec) = playlist {
                    let mut msg = edit_msg(ctx, interaction, &format!("Found {} songs", vec.len())).await;
                    let len = vec.len();
                    let mut succes = 0; 
                    let mut failed = 0;
                    for (index, song) in vec.into_iter().enumerate() {
                        let source = match song.contains("https://") {
                            true => match rustube::rustube(song, true).await {
                                Ok(src) => src,
                                Err(_) => {
                                    failed += 1;
                                    continue
                                },
                            },
                            false => match rustube::rustube_search(song, true).await {
                                Ok(src) => src,
                                Err(_) => {
                                    failed += 1;
                                    continue
                                }
                            },
                        };

                        let (track, track_handle) = create_player(source.into());
                        track_handle
                            .add_event(
                                Event::Periodic(Duration::from_secs(0), None),
                                SongStart::new(interaction.channel_id, ctx.http.clone()),
                            )
                            .unwrap();
                        
                        let metadata = track_handle.metadata().clone();
                        succes += 1;
                        let msg_content = format!("`Loading... {}/{}`\n`Ok: {} | Failed: {}`", index+1, len, succes , failed);
                        let content = get_msg(metadata, user);
                        trace!("{content}");

                        let mut handler = handler_lock.lock().await;
                        handler.join(voice_channel).await.unwrap();
                        handler.enqueue(track);
                        msg.edit(ctx.http.clone(), |m| m.content(msg_content)).await.unwrap();
                    }
                } else {
                    edit_msg(ctx, interaction, "Invalid url").await;
                }
            } else if let Ok(source) = rustube::rustube(uri, true).await {
                let (track, track_handle) = create_player(source.into());
                track_handle
                    .add_event(
                        Event::Periodic(Duration::from_secs(0), None),
                        SongStart::new(interaction.channel_id, ctx.http.clone()),
                    )
                    .unwrap();

                let metadata = track_handle.metadata().clone();
                let content = get_msg(metadata, user);
                trace!("{content}");

                let mut handler = handler_lock.lock().await;
                handler.join(voice_channel).await.unwrap();
                handler.enqueue(track);

                edit_msg(ctx, interaction, &content).await;
            } else {
                edit_msg(ctx, interaction, "Nothing found").await;
            }
        } else if let Ok(source) = rustube::rustube_search(uri, true).await {
            let (track, track_handle) = create_player(source.into());
            track_handle
                .add_event(
                    Event::Periodic(Duration::from_secs(0), None),
                    SongStart::new(interaction.channel_id, ctx.http.clone()),
                )
                .unwrap();

            let metadata = track_handle.metadata().clone();
            let content = get_msg(metadata, user);
            trace!("{content}");

            let mut handler = handler_lock.lock().await;
            handler.join(voice_channel).await.unwrap();
            handler.enqueue(track);

            edit_msg(ctx, interaction, &content).await;
        } else {
            edit_msg(ctx, interaction, "Nothing found").await;
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

fn get_msg(metadata: Metadata, user: &User) -> String {
    let content = format!(
        "{} added `{}` by `{}`",
        user.name,
        metadata.title.unwrap(),
        match metadata.artist {
            Some(name) => name,
            None => metadata.channel.unwrap(),
        }
    );
    content
}
