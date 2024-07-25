use std::time::Duration;

use poise::{
    serenity_prelude::{
        self as serenity, ComponentInteraction, ComponentInteractionData,
        ComponentInteractionDataKind::Button, Context as EvtCtx, CreateInputText,
        CreateInteractionResponseFollowup, CreateQuickModal, FullEvent::*, InputTextStyle::Short,
        Interaction::Component,
    },
    FrameworkContext,
};

use crate::{done, error, info, search::search_message, Color, Data, Death, Error, Res};

/// The event handler or dispatcher for serenity event.
pub async fn handler(
    ctx: &EvtCtx,
    event: &serenity::FullEvent,
    _: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Res {
    let res: Res = match event {
        Ready {
            data_about_bot: serenity::Ready { user, .. },
        } => {
            done!(
                "Bot is ready. Login as {}",
                format!("{}#{}", user.name, user.discriminator.unwrap()).green()
            );
            Ok(())
        }

        Message { new_message: msg } if msg.author.id != ctx.cache.current_user().id => {
            search_message(ctx, msg, data).await
        }

        // handle button shit
        InteractionCreate {
            interaction:
                Component(
                    interaction @ ComponentInteraction {
                        data:
                            ComponentInteractionData {
                                custom_id,
                                kind: Button,
                                ..
                            },
                        ..
                    },
                ),
        } => button_handler(interaction, ctx, custom_id, data).await,

        _ => Ok(()),
    };

    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            error!(
                "Cannot handle {} event due to: {err}",
                event.snake_case_name()
            );
            Err(err)
        }
    }
}

async fn button_handler(
    interaction: &ComponentInteraction,
    ctx: &EvtCtx,
    custom_id: &str,
    data: &Data,
) -> Res {
    match custom_id {
        "remove_cache" => cache_remove(interaction, ctx, data).await,
        "retry" => retry(),
        _ => Ok(()),
    }
}

async fn cache_remove(interaction: &ComponentInteraction, ctx: &EvtCtx, data: &Data) -> Res {
    info!("Cache removal request receive...");
    info!("Asking for which cache to remove...");

    let res = interaction
        .quick_modal(
            ctx,
            CreateQuickModal::new("Remove Cache")
                .timeout(Duration::from_secs(60))
                .field(
                    CreateInputText::new(Short, "Portrait Hash", "")
                        .placeholder("If you don't know what this is don't touch it"),
                ),
        )
        .await?
        .unwrap();

    done!("Answer received");

    res.interaction.defer(&ctx.http).await?;

    let hash: u64 = res.inputs.first().unwrap().parse().unwrap();

    info!("Request to remove cache for hash {}", hash.red());
    info!("Checking caches...");

    let res = {
        data.cache
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
        data.save_cache();
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
fn retry() -> Res {
    todo!()
}
