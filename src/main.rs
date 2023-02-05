use std::{collections::HashMap, fs::read_to_string, time::Duration};

use serenity::{
    async_trait,
    model::{application::interaction::Interaction, gateway::Ready, prelude::GuildId},
    prelude::{Client, Context, EventHandler, GatewayIntents},
};

use once_cell::sync::Lazy;
use songbird::SerenityInit;

mod commands;

static DISCORD_DATA: Lazy<serde_json::Value> = Lazy::new(|| {
    let json = read_to_string("./config.json").unwrap();
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&json).unwrap();
    data.get("discord").unwrap().to_owned()
});

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
                _ => panic!(),
            };
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("{} is connected!", ready.user.name);

        let guilds_ids = DISCORD_DATA.get("guilds_ids").unwrap().as_array().unwrap();

        println!("Added guilds:");
        for guild_id in guilds_ids {
            let guild_id = GuildId(guild_id.as_u64().unwrap());
            let guild_name = guild_id
                .name(&ctx.cache)
                .unwrap_or_else(|| "None".to_string());
            println!("  {} - {},", guild_name, guild_id);

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
    let token = DISCORD_DATA.get("token").unwrap().as_str().unwrap();

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
