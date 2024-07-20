#![allow(missing_docs)]

use magpie_tutor::{debug, done, error, info, Color};
use magpie_tutor::{query::query_message, CmdCtx, Data, Error, Res};
use poise::serenity_prelude::{
    self as serenity, Context as EvtCtx, CreateEmbed, FullEvent::*, GatewayIntents,
};
use poise::{CreateReply, FrameworkContext};

/// Test command
#[poise::command(slash_command)]
async fn test(ctx: CmdCtx<'_>) -> Res {
    let mut msg = CreateReply::default();

    for _ in (0..15) {
        msg = msg.embed(CreateEmbed::new().title("Embed"));
    }

    ctx.send(msg).await?;

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
                info!("Refreshing commands...");
                // Clear all command
                //poise::builtins::register_globally::<Data, Error>(ctx, &[]).await?;

                // Register all the normal command
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                done!(
                    "Finish registering {} commands",
                    framework.options().commands.len().green()
                );

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
    ctx: &EvtCtx,
    event: &serenity::FullEvent,
    _: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Res {
    let res: Res = match event {
        Message { new_message: msg } if msg.author.id != ctx.cache.current_user().id => {
            query_message(ctx, msg, data).await
        }
        Ready {
            data_about_bot: serenity::Ready { user, .. },
        } => {
            done!(
                "Bot is ready. Login as {}",
                format!("{}#{}", user.name, user.discriminator.unwrap()).red()
            );
            Ok(())
        }
        _ => Ok(()),
    };

    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            error!("Cannot handle {} event due to:", event.snake_case_name());
            error!("{err}");
            Err(err)
        }
    }
}
