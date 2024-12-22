use poise::serenity_prelude::{ChannelId, Context, GuildId, Message};

use crate::Res;

pub async fn message_handler(msg: &Message, ctx: &Context) -> Res {
    if msg.content.starts_with("what") {
        let content = desc_faq(msg.content.to_lowercase().as_str());
        if !content.is_empty() {
            msg.reply(ctx, content).await?;
        }
    } else if msg.content.contains("want to play")
        || msg.content.contains("want to fight")
            && msg
                .guild_id
                .is_some_and(|id| id == GuildId::new(994573431880286289))
            && msg.channel_id != ChannelId::new(1065751579485032629)
    {
        msg.reply(ctx, "
You seem to be asking for a game in the the wrong channel!
You can look at [this faq](https://discord.com/channels/994573431880286289/1168644586319659100/1181115229610983424), or:
- Host a room in the game
- Go to the <#1065751579485032629> channel
- Choose a inactive lobby (choose one that no one is talking in). Competive lobby usually entail harder and more meta gameplay.
- Send a message with the room code and ping the `Gamer (PING IF LFG)` role"
        ).await?;
    }
    Ok(())
}

fn desc_faq(what: &str) -> &'static str {
    match what {
        "what is link" | "what is <:cost_link:1240999261831958599>" | "what are links" => "
Links are an alternate cost type in Descryption. This cost type predominantly appears on Artistry cards. 

Links work as follows:
- Whenever a card is played in any way, it yields 1 link to its owner.
- Cards which cost links expend that many links as they are being played. (They then still yield the normal 1.)
- All links are lost whenever your turn ends. Links yielded to you during your opponent's turn will be available to spend on your next turn. ",

"what is heat" | "what is <:cost_heat:1099344819492495451>" | "what are heats" => "
Heats are an alernate cost type in IMR (Inscryption Multiplayer Redux). You gain heats when a card is discarded from your hand. Unspent heat are kept across turn.",

"what is sap" | "what is <:cost_sap:1125555492853403708>" | "what are saps"=> "
Saps are an alternate cost type in IMR (Inscryption Multiplayer Redux). Saps function identical to blood only you can also sacrifice bloodless card for saps.",

        _ => ""
    }
}
