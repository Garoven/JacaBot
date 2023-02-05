use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use songbird::tracks::LoopState;

use super::send_msg;

pub async fn run(interaction: &ApplicationCommandInteraction, ctx: &Context) {
    let cache = &ctx.cache;

    let guild_id = interaction.guild_id.expect("Not in channel");
    let guild = cache.guild(guild_id).unwrap();

    let user_id = interaction.user.id;

    let voice_channel = guild
        .voice_states
        .get(&user_id)
        .and_then(|voice_state| voice_state.channel_id);

    if voice_channel.is_none() {
        return send_msg(ctx, interaction, "Not in voice channel").await;
    }

    let menager = songbird::get(ctx)
        .await
        .expect("Failed to get manager")
        .clone();

    let handler_lock = match menager.get(guild_id) {
        Some(handler) => handler,
        None => return send_msg(ctx, interaction, "Nothing to loop").await,
    };
    let handler = handler_lock.lock().await;

    if handler.current_channel().is_none() {
        return send_msg(ctx, interaction, "Nothing to loop").await;
    }

    if voice_channel.unwrap().0 == handler.current_channel().unwrap().0 {
        if let Some(track_handle) = handler.queue().current() {
            match track_handle.get_info().await.unwrap().loops {
                LoopState::Infinite => {
                    track_handle.disable_loop().unwrap();
                    send_msg(ctx, interaction, "Repeat disabled").await
                }
                _ => {
                    track_handle.enable_loop().unwrap();
                    send_msg(ctx, interaction, "Repeat enabled").await
                }
            }
        } else {
            send_msg(ctx, interaction, "Nothing to loop").await
        }
    } else {
        send_msg(ctx, interaction, "Not in bot voice channel").await
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("repeat").description("pause current repeat")
}
