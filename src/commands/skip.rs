use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

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
        None => return send_msg(ctx, interaction, "Nothing to skip").await,
    };
    let handler = handler_lock.lock().await;

    if handler.current_channel().is_none() {
        return send_msg(ctx, interaction, "Nothing to skip").await;
    }

    if voice_channel.unwrap().0 == handler.current_channel().unwrap().0 {
        if handler.queue().current().is_some() {
            handler.queue().skip().unwrap();
            send_msg(ctx, interaction, "Song skipped").await
        } else {
            send_msg(ctx, interaction, "Nothing to skip").await
        }
    } else {
        send_msg(ctx, interaction, "Not in bot voice channel").await
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("skip")
        .description("skips current song if exist")
}
