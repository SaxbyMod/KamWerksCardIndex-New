use poise::{
    serenity_prelude::{
        self as serenity, ComponentInteraction, ComponentInteractionData,
        ComponentInteractionDataKind::Button, Context as EvtCtx, FullEvent::*,
        Interaction::Component,
    },
    FrameworkContext,
};

use crate::{done, error, search::search_message, Color, Data, Error, Res};

mod button;
use button::button_handler;

/// The event handler or dispatcher for serenity event.
pub async fn handler(
    ctx: &EvtCtx,
    event: &serenity::FullEvent,
    _: FrameworkContext<'_, Data, Error>,
    _: &Data,
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
            search_message(ctx, msg, msg.guild_id.unwrap()).await
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
        } => button_handler(interaction, ctx, custom_id).await,

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
