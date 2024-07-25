//! Contain the main search function and implementations.
use std::vec;

use magpie_engine::bitsflag;
use poise::serenity_prelude::{
    colours::roles,
    ButtonStyle::{Danger, Primary},
    Context,
    CreateActionRow::Buttons,
    CreateAttachment, CreateButton, CreateEmbed, CreateMessage, Message,
};

use crate::{
    done, get_portrait, hash_card_url,
    helper::{current_epoch, fuzzy_best, FuzzyRes},
    info,
    query::query_message,
    resize_img, CacheData, Card, Color, Data, Death, MessageCreateExt, Res, CACHE_REGEX,
    DEBUG_CARD, SEARCH_REGEX, SETS,
};

mod embed;
use embed::{gen_embed, missing_embed};

bitsflag! {
    struct Modifier: u8 {
        QUERY = 1;
        ALL_SET = 1 << 1;
        DEBUG = 1 << 2;
    }
}

/// Main searching function.
pub async fn search_message(ctx: &Context, msg: &Message, data: &Data) -> Res {
    if !SEARCH_REGEX.is_match(&msg.content) {
        return Ok(());
    }
    info!(
        "Message with {} by {} seaching time",
        msg.content.red(),
        msg.author.name.magenta()
    );
    let start = std::time::Instant::now();
    let mut embeds = vec![];
    let mut attachment: Vec<CreateAttachment> = vec![];
    for (modifier, set_code, search_term) in SEARCH_REGEX.captures_iter(&msg.content).map(|c| {
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
                    'd' => t |= Modifier::DEBUG,
                    _ => (),
                }
            }

            if msg.content.contains(':') {
                t |= Modifier::QUERY;
            }

            t
        };

        let mut sets = vec![];
        if modifier.contains(Modifier::ALL_SET) {
            sets.extend(SETS.values());
        } else {
            for set in set_code.split('|') {
                if let Some(set) = SETS.get(set) {
                    sets.push(set);
                }
            }
        }

        sets.is_empty().then(|| sets.push(SETS.get("com").unwrap())); // put in a default set

        if modifier.contains(Modifier::QUERY) {
            embeds.push(query_message(sets, search_term));
            continue;
        }

        for set in sets {
            let FuzzyRes { rank, data: card } = if search_term == "old_data" {
                FuzzyRes {
                    rank: 4.2,
                    data: &*DEBUG_CARD,
                }
            } else if let Some(best) =
                fuzzy_best(search_term, set.cards.iter().collect(), 0.5, |c: &Card| {
                    c.name.as_str()
                })
            {
                best
            } else {
                embeds.push(missing_embed(search_term));
                continue;
            };

            if modifier.contains(Modifier::DEBUG) {
                embeds.push(
                    CreateEmbed::new()
                        .color(roles::BLUE)
                        .description(format!("```\n{card:#?}\n```")),
                );
                continue;
            }

            let mut embed = gen_embed(rank, card, SETS.get(card.set.code()).unwrap());
            let hash = hash_card_url(card);
            let mut cache_guard = data.cache.lock().unwrap_or_die("Cannot lock cache");

            match cache_guard.get(&hash) {
                Some(CacheData {
                    channel_id,
                    attachment_id,
                    expire_date,
                }) if current_epoch() >= *expire_date as u128 => {
                    embed = embed.thumbnail(format!("https://cdn.discordapp.com/attachments/{channel_id}/{attachment_id}/{hash}.png"));
                }
                option => {
                    // remove the cache when the thing expire
                    if option.is_some() {
                        info!("Cache for {} have expire removing...", hash.blue());
                        cache_guard.remove(&hash);
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
                .components(vec![Buttons(vec![
                    CreateButton::new("remove_cache")
                        .style(Danger)
                        .label("Remove Cache"),
                    CreateButton::new("retry").style(Primary).label("Retry"),
                ])])
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
    let mut cache_guard = data.cache.lock().unwrap_or_die("Cannot lock cache");
    for url in msg
        .embeds
        .iter()
        .filter_map(|e| e.thumbnail.as_ref().map(|e| &e.url))
    {
        let capture: [&str; 4] = CACHE_REGEX
            .captures(url)
            .unwrap_or_else(|| panic!("Cannot find a match in url: {url}"))
            .extract()
            .1;

        let filename = capture[2].parse().unwrap();
        let cache_data = CacheData {
            channel_id: capture[0]
                .parse()
                .unwrap_or_else(|_| panic!("Cannot parse channel id: {}", capture[0])),
            attachment_id: capture[1]
                .parse()
                .unwrap_or_else(|_| panic!("Cannot parse attachment id: {}", capture[1])),
            expire_date: u64::from_str_radix(capture[3], 16)
                .unwrap_or_else(|_| panic!("Cannot parse expire date: {}", capture[3])),
        };

        if cache_guard.get(&filename).is_some() {
            info!("Cache for {} found skipping...", filename.blue());
            continue;
        }

        // Insert in the new cache replacing the old one
        if cache_guard.insert(filename, cache_data).is_none() {
            done!(
                "{} cache for card hash {}",
                "Create".green(),
                filename.blue()
            );
            new_cache += 1;
        };
    }

    if new_cache > 0 {
        done!("{} new cache(s) found", new_cache.green());
        info!("Saving caches...");

        // unlock the cache to avoid deadlock
        drop(cache_guard);

        // save the updated cache
        data.save_cache();
    } else {
        done!("No new caches found! Nothing to update :3");
    }
    Ok(())
}
