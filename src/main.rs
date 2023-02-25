use std::time::Duration;

use serenity::{
    async_trait,
    model::{application::interaction::Interaction, gateway::Ready, prelude::{GuildId, Message, RoleId, ReactionType, EmojiId}},
    prelude::{Client, Context, EventHandler, GatewayIntents},
};

use songbird::SerenityInit;

mod config;
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
            msg.react(ctx.http.clone(), ReactionType::Custom { animated: false, id: EmojiId::from(1050836873762852974), name: Some("sus".to_string()) }).await.unwrap();
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        tokio::time::sleep(Duration::from_secs(2)).await;
        log(&format!("{} is connected!", ready.user.name), Log::Info());

        let guilds_ids = config::DISCORD_CONFIG.get("guilds_ids").unwrap().as_array().unwrap();

        log(&format!("Added {} guilds", guilds_ids.len()), Log::Info());
        for guild_id in guilds_ids {
            let guild_id = GuildId(guild_id.as_u64().unwrap());

            GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
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
            .unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let token = config::DISCORD_CONFIG.get("token").unwrap().as_str().unwrap();

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log(&format!("Client error: {why:?}"), Log::Error());
    }
}

enum Log {
    Info(),
    Warn(),
    Error()
}

impl Log {
    fn get(&self) -> String {
        match &self {
            Log::Info() => "Info".to_string(),
            Log::Warn() => "Warn".to_string(),
            Log::Error() => "Error".to_string(),
        }
    }
}

fn log(msg: &str, log_type: Log) {
    let time = chrono::Local::now().format("%d-%m-%Y %H:%M:%S");
    println!("[{time}][{}] {msg}", log_type.get())
}
