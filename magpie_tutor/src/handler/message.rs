use poise::serenity_prelude::{Context, Message};

use crate::Res;

pub async fn message_handler(msg: &Message, ctx: &Context) -> Res {
    if msg.content.starts_with("what") {
        let content = desc_faq(msg.content.to_lowercase().as_str());
        if !content.is_empty() {
            msg.reply(ctx, content).await?;
        }
    }
    Ok(())
}

fn desc_faq(what: &str) -> &'static str {
    match what {
        "what is link" | "what is <:cost_link:1240999261831958599>" => "
Links are an alternate cost type in Descryption. This cost type predominantly appears on Artistry cards. 

Links work as follows:
- Whenever a card is played in any way, it yields 1 link to its owner.
- Cards which cost links expend that many links as they are being played. (They then still yield the normal 1.)
- All links are lost whenever your turn ends. Links yielded to you during your opponent's turn will be available to spend on your next turn. ",

"what is heat" | "what is <:cost_heat:1099344819492495451>" => "
Heats are an alernate cost type in IMR (Inscryption Multiplayer Redux). You gain heats when a card is discarded from your hand. Unspent heat are kept across turn.",

"what is sap" | "what is <:cost_sap:1125555492853403708>" => "
Saps are an alternate cost type in IMR (Inscryption Multiplayer Redux). Saps function identical to blood only you can also sacrifice bloodless card for saps.",

        _ => ""
    }
}
