use magpie_tutor::{query::query_message, Context, Data, Error, Res};
use poise::serenity_prelude::{self as serenity, FullEvent::*, GatewayIntents, GuildId};
use poise::FrameworkContext;

const CLIENT_ID: u64 = 1255931136044171285;

/// Test command
#[poise::command(slash_command)]
async fn test(ctx: Context<'_>) -> Res {
    ctx.say("This is a test command").await?;

    ctx.data().save_cache();

    Ok(())
}

// main entry point of the bot
#[tokio::main]
async fn main() {
    // your token need to be in the enviroment variable
    let token = std::env::var("TUTOR_TOKEN").expect("missing token in env var");
    let intents = GatewayIntents::privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // poise framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![test()],
            event_handler: |ctx, event, fw, data| Box::pin(handler(ctx, event, fw, data)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(1199457939333849118),
                )
                .await?;

                Ok(Data::new())
            })
        })
        .build();

    // client time
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Res {
    match event {
        Message { new_message: msg } if msg.author.id.get() != CLIENT_ID => {
            query_message(ctx, msg, data).await
        }
        _ => Ok(()),
    }
}
