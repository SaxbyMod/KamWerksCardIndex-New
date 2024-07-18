use crate::embed::gen_embed;
use crate::fuzzy::{fuzzy_rank, FuzzyRes};
use crate::{get_portrait, hash_str, resize_img, CacheData, Card, Data, MessageCreateExt, Res};
use poise::serenity_prelude::{Context, CreateAttachment, CreateMessage, Message};
use std::time::{SystemTime, UNIX_EPOCH};

// main querying function
pub async fn query_message(ctx: &Context, msg: &Message, data: &Data) -> Res {
    if !data.query_regex.is_match(&msg.content) {
        return Ok(());
    }
    println!("CARD TIME :3333");
    let start = std::time::Instant::now();
    let mut embeds = vec![];
    let mut attacment = vec![];
    for (modifier, set_code, card_name) in data.query_regex.captures_iter(&msg.content).map(|c| {
        (
            c.get(1).map_or("", |s| s.as_str()),
            c.get(2).map_or("", |s| s.as_str()),
            c.get(3).map_or("", |s| s.as_str()),
        )
    }) {
        if !modifier.is_empty() {
            return Ok(());
        }

        let mut sets = vec![];
        for set in set_code.split('|') {
            if let Some(set) = data.sets.get(set) {
                sets.push(set);
            }
        }

        sets.is_empty()
            .then(|| sets.push(data.sets.get("com").unwrap())); // put in a default set

        let fuzzy_res = if card_name == "old_data" {
            FuzzyRes {
                rank: 4.2,
                data: &data.debug_card,
            }
        } else {
            fuzzy_rank(
                card_name,
                sets.iter().flat_map(|s| s.cards.iter()).collect(),
                |c: &Card| c.name.as_str(),
            )
            .0
        };

        let mut embed = gen_embed(
            &fuzzy_res,
            data.sets.get(fuzzy_res.data.set.code()).unwrap(),
        );

        let hash = hash_str(&fuzzy_res.data.name);

        match data
            .portrait_cache
            .lock()
            .unwrap()
            .get(&hash)
        {
            Some(CacheData {channel_id, attachment_id, expiry_date: expire_date})
                // check if the link have expire
                if SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Are you Marty McFly? Please return to the correct timeline")
                    .as_millis()
                    >= *expire_date as u128 =>
            {
                embed = embed.thumbnail(format!("https://cdn.discordapp.com/attachments/{channel_id}/{attachment_id}/{hash}.png"));
            }
            option => {
                // remove the cache when the thing expire
                if option.is_some() {
                    data.remove_cache(hash);
                }
                attacment.push(CreateAttachment::bytes(
                    resize_img(get_portrait(&fuzzy_res.data.portrait), 2),
                    hash.to_string() + ".png",
                ));
            }
        }

        embeds.push(embed);
    }
    let msg = msg
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .content(format!("Search completed in {:.1?}", start.elapsed()))
                .embeds(embeds)
                .files(attacment)
                .reply(msg),
        )
        .await?;

    for url in msg
        .embeds
        .iter()
        .map(|e| &e.thumbnail.as_ref().unwrap().url)
    {
        let t: (u64, CacheData) = {
            let t: [&str; 4] = data
                .cache_regex
                .captures(url)
                .unwrap_or_else(|| panic!("Cannot find a match in url: {url}"))
                .extract()
                .1;

            (
                t[2].parse().unwrap(),
                CacheData {
                    channel_id: t[0]
                        .parse()
                        .unwrap_or_else(|_| panic!("Cannot parse channel id: {}", t[0])),
                    attachment_id: t[1]
                        .parse()
                        .unwrap_or_else(|_| panic!("Cannot parse attachment id: {}", t[1])),
                    expiry_date: u64::from_str_radix(t[3], 16)
                        .unwrap_or_else(|_| panic!("Cannot parse expriry date: {}", t[3])),
                },
            )
        };

        data.insert_cache(t.0, t.1);
    }

    data.save_cache(); // save the updated cache

    Ok(())
}
