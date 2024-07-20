//! Contain the main query function and implementation.
use crate::embed::{gen_embed, missing_embed};
use crate::fuzzy::{fuzzy_best, FuzzyRes};
use crate::{
    get_portrait, hash_card_url, resize_img, CacheData, Card, Color, Data, Death, MessageCreateExt,
    Res,
};
use magpie_engine::bitsflag;
use poise::serenity_prelude::{Context, CreateAttachment, CreateMessage, Message};
use std::time::{SystemTime, UNIX_EPOCH};

bitsflag! {
    struct Modifier: u8 {
        QUERY = 1;
        ALL_SET = 1 << 1;
    }
}

/// main querying function.
pub async fn query_message(ctx: &Context, msg: &Message, data: &Data) -> Res {
    if !data.query_regex.is_match(&msg.content) {
        return Ok(());
    }
    info!(
        "Message with {} by {} querying time",
        msg.content.red(),
        msg.author.name.magenta()
    );
    let start = std::time::Instant::now();
    let mut embeds = vec![];
    let mut attachment: Vec<CreateAttachment> = vec![];
    for (modifier, set_code, card_name) in data.query_regex.captures_iter(&msg.content).map(|c| {
        (
            c.get(1).map_or("", |s| s.as_str()),
            c.get(2).map_or("", |s| s.as_str()),
            c.get(3).map_or("", |s| s.as_str()),
        )
    }) {
        let modifier = {
            let mut t = Modifier::EMPTY;
            for m in modifier.chars() {
                match m {
                    'q' => t |= Modifier::QUERY,
                    '*' => t |= Modifier::ALL_SET,
                    _ => (),
                }
            }
            t
        };

        let mut sets = vec![];
        if modifier.contains(Modifier::ALL_SET) {
            sets.extend(data.sets.values());
        } else {
            for set in set_code.split('|') {
                if let Some(set) = data.sets.get(set) {
                    sets.push(set);
                }
            }
        }

        sets.is_empty()
            .then(|| sets.push(data.sets.get("com").unwrap())); // put in a default set

        for set in sets {
            let FuzzyRes { rank, data: card } = if card_name == "old_data" {
                FuzzyRes {
                    rank: 4.2,
                    data: &data.debug_card,
                }
            } else if let Some(best) =
                fuzzy_best(card_name, set.cards.iter().collect(), 0.5, |c: &Card| {
                    c.name.as_str()
                })
            {
                best
            } else {
                embeds.push(missing_embed(card_name));
                continue;
            };

            let mut embed = gen_embed(rank, card, data.sets.get(card.set.code()).unwrap());

            let hash = hash_card_url(card);

            let mut cache = data.cache.lock().unwrap_or_die("Cannot lock cache");

            match cache.get(&hash) {
                Some(CacheData {channel_id, attachment_id, expire_date})
                    // check if the link have expire if it is go make a new one
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
                        info!("Cache for {} have expire removing...", hash.blue());
                        cache.remove(&hash);
                        done!("{} cache for card hash {}", "Remove".red(), hash.blue());
                    }

                    let filename = hash.to_string() + ".png";

                    if !attachment.iter().any(|a| a.filename == filename) {
                        attachment.push(CreateAttachment::bytes(
                            resize_img(get_portrait(&card.portrait), 2),
                            filename,
                        ));
                    }
                }
            }

            embeds.push(embed);
        }
    }

    let msg = msg
        .channel_id
        .send_files(
            &ctx.http,
            attachment,
            CreateMessage::new()
                .content(format!("Search completed in {:.1?}", start.elapsed()))
                .embeds(embeds)
                .reply(msg),
        )
        .await?;

    // Update the cache
    //
    // We always do this because.
    // 1. It doesn't take too long and it doesn't affect other thing
    // 2. The cache might have expire and we need to record that
    info!("Updating caches...");
    let mut new_cache = 0;
    let mut cache = data.cache.lock().unwrap_or_die("Cannot lock cache");
    for url in msg
        .embeds
        .iter()
        .filter_map(|e| e.thumbnail.as_ref().map(|e| &e.url))
    {
        let t: (u64, CacheData) = {
            let t: [&str; 4] = data
                .cache_regex
                .captures(url)
                .unwrap_or_else(|| panic!("Cannot find a match in url: {url}"))
                .extract()
                .1;

            (
                t[2].parse().unwrap(), // the file name or the card name hash
                CacheData {
                    channel_id: t[0]
                        .parse()
                        .unwrap_or_else(|_| panic!("Cannot parse channel id: {}", t[0])),
                    attachment_id: t[1]
                        .parse()
                        .unwrap_or_else(|_| panic!("Cannot parse attachment id: {}", t[1])),
                    expire_date: u64::from_str_radix(t[3], 16)
                        .unwrap_or_else(|_| panic!("Cannot parse expire date: {}", t[3])),
                },
            )
        };

        if cache.get(&t.0).is_some() {
            info!("Cache for {} found skipping...", t.0.blue());
            continue;
        }

        // Insert in the new cache replacing the old one
        if cache.insert(t.0, t.1).is_none() {
            done!("{} cache for card hash {}", "Create".green(), t.0.blue());
            new_cache += 1;
        };
    }

    if new_cache > 0 {
        done!("{} new cache(s) found", new_cache.green());
        info!("Saving caches...");
        // unlock the cache so we can save
        drop(cache);

        // save the updated cache
        data.save_cache();
    } else {
        done!("No new caches found! Nothing to update :3");
    }
    Ok(())
}
