use poise::serenity_prelude::{
    CreateActionRow, CreateAllowedMentions, CreateAttachment, CreateEmbed,
    CreateInteractionResponseMessage, CreateMessage, EditAttachments, EditInteractionResponse,
    InteractionResponseFlags, MessageFlags,
};

use crate::{builder, debug};

builder! {
    /// Message adapter to conver between various message type
    #[derive(Debug)]
    pub struct MessageAdapter {
        /// The content for this message
        pub content: String,
        /// The embeds on this message
        pub embeds: Vec<CreateEmbed>,
        /// The attachments or files of this message
        pub attachments: Vec<CreateAttachment>,
        /// Which user or group is allow to be mention by this message
        pub allowed_mentions: CreateAllowedMentions,
        /// Component attached to this message
        pub components: Vec<CreateActionRow>,
        /// Wherever this message is ephemeral or only visible to only the user who invoke the
        /// interaction
        pub ephemeral: bool,
    }
}

impl From<MessageAdapter> for CreateMessage {
    fn from(
        MessageAdapter {
            content,
            embeds,
            attachments,
            allowed_mentions,
            components,
            ephemeral,
        }: MessageAdapter,
    ) -> Self {
        let mut flags = MessageFlags::default();
        flags.set(MessageFlags::EPHEMERAL, ephemeral);

        CreateMessage::new()
            .content(content)
            .embeds(embeds)
            .files(attachments)
            .allowed_mentions(allowed_mentions)
            .components(components)
            .flags(flags)
    }
}

impl From<MessageAdapter> for EditInteractionResponse {
    fn from(
        MessageAdapter {
            content,
            embeds,
            attachments,
            allowed_mentions,
            components,
            ..
        }: MessageAdapter,
    ) -> Self {
        let mut new_attach = EditAttachments::new();
        for a in attachments {
            new_attach = new_attach.add(a);
        }

        EditInteractionResponse::new()
            .content(content)
            .embeds(embeds)
            .attachments(new_attach)
            .allowed_mentions(allowed_mentions)
            .components(components)
    }
}

impl From<MessageAdapter> for CreateInteractionResponseMessage {
    fn from(
        MessageAdapter {
            content,
            embeds,
            attachments,
            allowed_mentions,
            components,
            ephemeral,
        }: MessageAdapter,
    ) -> Self {
        let mut flags = InteractionResponseFlags::default();
        flags.set(InteractionResponseFlags::EPHEMERAL, ephemeral);

        CreateInteractionResponseMessage::new()
            .content(content)
            .embeds(embeds)
            .files(attachments)
            .allowed_mentions(allowed_mentions)
            .components(components)
            .flags(flags)
    }
}
