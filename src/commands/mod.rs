use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};

pub mod pause;
pub mod ping;
pub mod play;
pub mod repeat;
pub mod resume;
pub mod skip;
pub mod stop;

async fn send_msg(ctx: &Context, interaction: &ApplicationCommandInteraction, content: &str) {
    if let Err(why) = interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

async fn edit_msg(ctx: &Context, interaction: &ApplicationCommandInteraction, content: &str) {
    if let Err(why) = interaction
        .edit_original_interaction_response(&ctx.http, |response| response.content(content))
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}
