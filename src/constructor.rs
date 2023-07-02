use twilight_http::Client;
use twilight_model::channel::{
    message::{MessageFlags, MessageType},
    Message,
};

use crate::{error::Error, not_last::Info, thread, MessageSource};

impl<'a> MessageSource<'a> {
    /// Create [`MessageSource`] from a [`Message`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceRichPresence`] if the message is related
    /// to rich presence, which can't be recreated by bots
    ///
    /// Returns [`Error::SourceAttachment`] if the message has an attachment,
    /// this will be handled more gracefully in the future
    ///
    /// Returns [`Error::SourceComponent`] if the message has a component, which
    /// would be broken since the components would then be sent to the cloner
    /// bot
    ///
    /// Returns [`Error::SourceReaction`] if the message has a reaction, this
    /// will be handled more gracefully in the future
    ///
    /// Returns [`Error::SourceSticker`] if the message has a sticker, which
    /// webhook messages can't have
    ///
    /// Returns [`Error::SourceThread`] if the message has a thread or forum
    /// post created from it, this will be handled more gracefully in the
    /// future
    ///
    /// Returns [`Error::SourceVoice`] if the message is a voice message, which
    /// bots currently can't create
    ///
    /// Returns [`Error::SourceSystem`] if the message's type isn't
    /// [`MessageType::Regular`] or [`MessageType::Reply`] or has role
    /// subscription data, which are edge-cases that can't be replicated
    /// correctly
    ///
    /// Returns [`Error::SourceContentInvalid`] if the message's content is
    /// invalid
    ///
    /// Returns [`Error::SourceUsernameInvalid`] if username of the message's
    /// author is invalid
    pub fn from_message(message: &'a Message, http: &'a Client) -> Result<Self, Error> {
        if message.activity.is_some() || message.application.is_some() {
            return Err(Error::SourceRichPresence);
        }
        if !message.attachments.is_empty() {
            return Err(Error::SourceAttachment);
        }
        if !message.components.is_empty() {
            return Err(Error::SourceComponent);
        }
        if !message.reactions.is_empty() {
            return Err(Error::SourceReaction);
        }
        if !message.sticker_items.is_empty() {
            return Err(Error::SourceSticker);
        }
        if message.thread.is_some()
            || message.id == message.channel_id.cast()
            || message
                .flags
                .is_some_and(|flags| flags.contains(MessageFlags::HAS_THREAD))
        {
            return Err(Error::SourceThread);
        }
        if message
            .flags
            .is_some_and(|flags| flags.contains(MessageFlags::IS_VOICE_MESSAGE))
        {
            return Err(Error::SourceVoice);
        }
        if !matches!(message.kind, MessageType::Regular | MessageType::Reply)
            || message.role_subscription_data.is_some()
        {
            return Err(Error::SourceSystem);
        }
        twilight_validate::message::content(&message.content)
            .map_err(|_| Error::SourceContentInvalid)?;
        twilight_validate::request::webhook_username(&message.author.name)
            .map_err(|_| Error::SourceUsernameInvalid)?;

        Ok(MessageSource {
            source_id: message.id,
            source_channel_id: message.channel_id,
            content: &message.content,
            embeds: &message.embeds,
            tts: message.tts,
            flags: message.flags,
            channel_id: message.channel_id,
            username: message
                .member
                .as_ref()
                .and_then(|member| member.nick.as_ref())
                .unwrap_or(&message.author.name),
            avatar_url: if let (Some(guild_id), Some(avatar)) = (
                message.guild_id,
                message.member.as_ref().and_then(|member| member.avatar),
            ) {
                format!(
                    "https://cdn.discordapp.com/guilds/{guild_id}/users/{}/avatars/{avatar}.{}",
                    message.author.id,
                    if avatar.is_animated() { "gif" } else { "png" }
                )
            } else if let Some(avatar) = message.author.avatar {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{avatar}.{}",
                    message.author.id,
                    if avatar.is_animated() { "gif" } else { "png" }
                )
            } else {
                format!(
                    "https://cdn.discordapp.com/embed/avatars/{}.png",
                    if message.author.discriminator == 0 {
                        (message.author.id.get() >> 22_u8) % 6
                    } else {
                        u64::from(message.author.discriminator % 5)
                    }
                )
            },
            webhook_name: "Message Cloner".to_owned(),
            thread_info: thread::Info::Unknown,
            webhook: None,
            later_messages: Info {
                messages: vec![],
                is_complete: false,
                is_source_created: false,
            },
            http,
        })
    }
}
