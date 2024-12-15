#![allow(missing_docs)]

use std::panic::PanicHookInfo;

use magpie_tutor::{
    done, error, frameworks, handler, info, CmdCtx, Color, Data, Res, CACHE, CACHE_FILE_PATH, SETS,
};
use poise::serenity_prelude::{CacheHttp, ClientBuilder, GatewayIntents, GuildId};

/// Test command
#[poise::command(slash_command)]
async fn test(ctx: CmdCtx<'_>) -> Res {
    ctx.say("Testing").await?;
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

/// Test to see if the IMF tunnel is online
#[poise::command(slash_command)]
async fn tunnel_status(ctx: CmdCtx<'_>) -> Res {
    ctx.defer().await?;
    ctx.say(match isahc::get("http://localtunnel.me") {
        Ok(_) => "Tunnel is up and running. If you have issue check out [this faq](https://discord.com/channels/994573431880286289/1168644586319659100/1168657617141366805).",
        Err(_) => "I cannot reach tunnel right now, this may mean tunnel is down but you can [check yourself](https://isitdownorjust.me/localtunnel-me/)."
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
    let framework = frameworks! {
        global: help(), show_modifiers();
        guild (1199457939333849118): test();
        guild (994573431880286289): tunnel_status();
        ---
        {
            Ok(Data::new())
        }
    };

    info!("Fetching set...");
    done!(
        "Finish fetching {} sets",
        SETS.lock().unwrap().len().green()
    );

    info!("Loading caches from {}...", CACHE_FILE_PATH.green());
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

fn panic_hook(info: &PanicHookInfo) {
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
