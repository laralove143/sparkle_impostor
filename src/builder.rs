use twilight_model::channel::{
    message::{MessageFlags, MessageType},
    Message,
};

use crate::{error::Error, MessageSource, ThreadInfo};

/// Defines how to clone messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct MessageSourceBuilder {
    /// See [`MessageSourceBuilder::ignore_threads`]
    pub ignore_threads: bool,
}

impl MessageSourceBuilder {
    /// Create a new, default builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ignore_threads: false,
        }
    }

    /// Don't handle the case of messages being in a thread
    ///
    /// Unless set, when the message doesn't contain thread info, it's received
    /// using the HTTP request
    ///
    /// When set and the message is in a thread, a few invalid HTTP requests
    /// will be made, it's not recommended to set this unless you know the
    /// message isn't in a thread
    #[must_use]
    pub const fn ignore_threads(mut self) -> Self {
        self.ignore_threads = true;
        self
    }

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
    /// Returns [`Error::SourceThread`] if the message has a thread created from
    /// it, this will be handled more gracefully in the future
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
    pub fn build_from_message(self, message: &Message) -> Result<MessageSource<'_>, Error> {
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
            thread_info: ThreadInfo::Unknown,
            builder: self,
        })
    }
}
