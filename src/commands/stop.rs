use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use super::send_msg;

pub async fn run(interaction: &ApplicationCommandInteraction, ctx: &Context) {
    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).unwrap();
    let menager = songbird::get(ctx)
        .await
        .expect("Failed to get manager")
        .clone();

    let handler_lock = match menager.get(guild_id) {
        Some(handler) => handler,
        None => return send_msg(ctx, interaction, "Bot is not connected to voice channel").await,
    };
    let handler = handler_lock.lock().await;

    let voice_channel = match guild
        .voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id) 
    {
        Some(v) => v.0,                                                                              
        None => return send_msg(ctx, interaction, "You are not connected to voice channel").await
    };
    let bot_voice_channel = match handler.current_channel() {
        Some(v) => v.0,
        None => return send_msg(ctx, interaction, "Bot is not connected to voice channel").await
        
    };

    if voice_channel == bot_voice_channel {
        handler.queue().stop();
        send_msg(ctx, interaction, "Cleared queue").await
    } else {
        send_msg(ctx, interaction, "You are not connected to voice channel with bot").await
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("stop")
        .description("Stop current song and clear queue")
}
