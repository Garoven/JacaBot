#![feature(once_cell)]
use log::{error, info, warn};
use std::{
    time::Duration,
    process::exit
};

use serenity::{
    async_trait,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        prelude::{GuildId, Message, RoleId, ReactionType, EmojiId}},
    prelude::{Client, Context, EventHandler, GatewayIntents},
};

use songbird::SerenityInit;

mod config;
mod errors;
mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "ping" => commands::ping::run(&command, &ctx).await,
                "play" => commands::play::run(&command, &ctx).await,
                "skip" => commands::skip::run(&command, &ctx).await,
                "stop" => commands::stop::run(&command, &ctx).await,
                "pause" => commands::pause::run(&command, &ctx).await,
                "resume" => commands::resume::run(&command, &ctx).await,
                "repeat" => commands::repeat::run(&command, &ctx).await,
                _ => {}
            };
        }
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mention_roles.contains(&RoleId::from(749235591299727461)) || msg.content.to_lowercase().contains("sus") || msg.content.to_lowercase().contains("among us") {
            if let Err(e) = msg.react(ctx.http.clone(), ReactionType::Custom { animated: false, id: EmojiId::from(1050836873762852974), name: Some("sus".to_string()) }).await {
                warn!("{e}");
            };
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        tokio::time::sleep(Duration::from_secs(2)).await;
        info!("{} is connected!", ready.user.name);

        let guilds_ids = match config::DISCORD_CONFIG.get("guilds_ids").and_then(|g| g.as_array()) {
            Some(g) => g,
            None => {
                warn!("Cannot get guilds_ids. Invaild config?");
                return;
            }
        };
        info!("Adding {} guilds", guilds_ids.len());
        for guild_id in guilds_ids {
            let guild_id = GuildId(match guild_id.as_u64() {
                Some(id) => id,
                None => {
                    warn!("Cannot get GuildId. Not a number?");
                    continue
                }
            });

            if let Err(e) = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| commands::ping::register(command))
                    .create_application_command(|command| commands::play::register(command))
                    .create_application_command(|command| commands::skip::register(command))
                    .create_application_command(|command| commands::stop::register(command))
                    .create_application_command(|command| commands::pause::register(command))
                    .create_application_command(|command| commands::resume::register(command))
                    .create_application_command(|command| commands::repeat::register(command))
            })
            .await
            {
                warn!("{e}");
            };
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let token = match config::DISCORD_CONFIG.get("token").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            error!("Cannot get token. No token in config file?");
            exit(1)
        }
    };

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = match Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    if let Err(why) = client.start().await {
        error!("Client error: {why}");
    }
}
