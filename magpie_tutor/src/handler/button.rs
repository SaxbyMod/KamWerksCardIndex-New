use std::time::Duration;

use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::{
    ComponentInteraction, Context, CreateInputText, CreateInteractionResponse::UpdateMessage,
    CreateInteractionResponseFollowup, CreateQuickModal, InputTextStyle::*,
};

use crate::search::process_search;
use crate::{done, info, save_cache, Color, Death, Res, CACHE};

pub async fn button_handler(
    interaction: &ComponentInteraction,
    ctx: &Context,
    custom_id: &str,
) -> Res {
    match custom_id {
        "remove_cache" => cache_remove(interaction, ctx).await,
        "retry" => retry(interaction, ctx).await,
        _ => Ok(()),
    }
}

async fn cache_remove(interaction: &ComponentInteraction, ctx: &Context) -> Res {
    info!("Cache removal request receive...");
    info!("Asking for which cache to remove...");

    let res = interaction
        .quick_modal(
            ctx,
            CreateQuickModal::new("Remove Cache")
                .timeout(Duration::from_secs(5))
                .field(
                    CreateInputText::new(Short, "Portrait Hash", "")
                        .placeholder("If you don't know what this do cancel this pop up."),
                ),
        )
        .await?;

    if res.is_none() {
        done!("Cache removal canceled");
        return Ok(());
    }

    let res = res.unwrap();

    done!("Answer received");

    res.interaction.defer(&ctx.http).await?;

    let hash: u64 = res.inputs.first().unwrap().parse().unwrap();

    info!("Request to remove cache for hash {}", hash.red());
    info!("Checking caches...");

    let res = {
        CACHE
            .lock()
            .unwrap_or_die("Cannnot lock cache")
            .remove(&hash)
    };

    if res.is_some() {
        done!("{} cache for card hash {}", "Removed".red(), hash.red());
        interaction
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Cache removed")
                    .ephemeral(true),
            )
            .await?;

        info!("Saving caches...");
        save_cache();
    } else {
        info!("Cache for card hash {} not found", hash.red());
        interaction
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Cache remove failed")
                    .ephemeral(true),
            )
            .await?;
        done!("Canceling removal");
    }

    Ok(())
}
async fn retry(interaction: &ComponentInteraction, ctx: &Context) -> Res {
    interaction
        .create_response(
            &ctx.http,
            UpdateMessage(
                process_search(
                    ctx.http()
                        .get_message(
                            interaction.message.channel_id,
                            interaction
                                .message
                                .message_reference
                                .as_ref()
                                .unwrap()
                                .message_id
                                .unwrap(),
                        )
                        .await?
                        .content
                        .as_str(),
                )
                .into(),
            ),
        )
        .await?;

    Ok(())
}
