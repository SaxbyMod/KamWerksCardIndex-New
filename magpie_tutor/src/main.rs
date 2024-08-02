#![allow(missing_docs)]

use std::panic::PanicInfo;

use magpie_tutor::{
    done, error, handler, info, CmdCtx, Color, Data, Error, Res, CACHE, CACHE_FILE, SETS,
};
use poise::{
    serenity_prelude::{ClientBuilder, CreateEmbed, GatewayIntents, GuildId},
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

/// Show help on what and how to use Magpie Tutor.
#[poise::command(slash_command)]
async fn help(ctx: CmdCtx<'_>) -> Res {
    ctx.say(
r#"
You can use Magpie to look up a card infomation by surrounding the card name in `[[]]`. A few "modifiers" can be added in front of the `[[]]` to change the output.

You can see these modifier by using the `/show-modifers` command. Set code are a special type of modifer that are 3 characters long and is at the end of the modifiers list and can be use to change the selected set.

For example:
- `[[stoat]]`: Look up the card name `stoat` using the server default set.
- `egg[[warren]]`: Look up the card name `warren` using the `egg` set.

"#,
    )
    .await?;

    Ok(())
}

macro_rules! mod_help {
    ($($code:ident: $code_desc:literal;)*---$($mod:literal: $desc:literal;)*) => {
        concat!(
            "# Set Codes\n",
            $(concat!("- `", stringify!($code), "`: ", $code_desc, ".\n"),)*
            "# Modifiers\n",
            $(concat!("- `", $mod,"`: ", $desc, " ", ".\n"),)*
        )
    };
}

/// Show the lists of all support modifiers and set code.
#[poise::command(slash_command)]
async fn show_modifiers(ctx: CmdCtx<'_>) -> Res {
    ctx.say(mod_help! {
        com: "IMF Competitive";
        egg: "Mr.Egg's Goofy";
        ete: "IMF Eternal";
        aug: "Augmented";
        ---
        "q": "Query instead of normal fuzzy search";
        "*": "Select all supported set";
        "d": "Output the raw data instead of embed";
        "c": "Output the embed in compact mode to save space";
        "\\`": "Skip this search match";

    })
    .await?;

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

    info!("Loading caches from {}...", CACHE_FILE.green());
    done!(
        "Finish loading {} caches",
        CACHE.lock().unwrap().len().green()
    );

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
            commands: vec![help(), show_modifiers()],
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

                poise::builtins::register_in_guild(
                    ctx,
                    &[test()],
                    GuildId::from(1199457939333849118),
                )
                .await?;

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
