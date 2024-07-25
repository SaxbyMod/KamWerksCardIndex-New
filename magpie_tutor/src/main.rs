#![allow(missing_docs)]

use std::panic::PanicInfo;

use magpie_tutor::{done, error, handler, info, CmdCtx, Color, Data, Error, Res, SETS};
use poise::{
    serenity_prelude::{ClientBuilder, CreateEmbed, GatewayIntents},
    CreateReply, Framework,
};

/// Test command
#[poise::command(slash_command)]
async fn test(ctx: CmdCtx<'_>) -> Res {
    let mut msg = CreateReply::default();

    for _ in 0..15 {
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
    let framework = build_framework();

    info!("Fetching set...");
    done!("Finish fetching {} sets", SETS.len().green());

    std::panic::set_hook(Box::new(panic_hook));

    // client time
    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

fn build_framework() -> Framework<Data, Error> {
    poise::Framework::builder()
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
        .build()
}

fn panic_hook(info: &PanicInfo) {
    if let Some(loc) = info.location() {
        error!(
            "Panic in file {} at line {}",
            loc.file().magenta(),
            loc.line().blue()
        );
    }
    let s = info
        .payload()
        .downcast_ref::<String>()
        .map(ToOwned::to_owned)
        .or_else(|| {
            info.payload()
                .downcast_ref::<&str>()
                .map(ToString::to_string)
        })
        .unwrap_or(String::new());

    let lines: Vec<_> = s.lines().collect();
    if lines.len() > 1 {
        error!("Panic message:");
        for l in lines {
            error!("{}", l.red());
        }
    } else {
        error!("Panic message: {}", s.red());
    }
}
