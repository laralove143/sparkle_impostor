use twilight_http::Client;
use twilight_model::{
    channel::{
        message::{MessageFlags, MessageType},
        Message,
    },
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    util::ImageHash,
};

use crate::{
    attachment, component, error::Error, later_messages, reaction, sticker, thread, MessageSource,
};

impl<'a> MessageSource<'a> {
    /// Create [`MessageSource`] from a [`Message`]
    ///
    /// # Warnings
    ///
    /// `message.guild_id` is usually `None` even if the message is in a guild,
    /// make sure this field is actually passed
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotInGuild`] if the message is not in a guild,
    ///
    /// Returns [`Error::RichPresence`] if the message is related
    /// to rich presence, which can't be recreated by bots
    ///
    /// Returns [`Error::Thread`] if the message has a thread or forum
    /// post created from it, this will be handled more gracefully in the
    /// future
    ///
    /// Returns [`Error::Voice`] if the message is a voice message, which
    /// bots currently can't create
    ///
    /// Returns [`Error::System`] if the message's type isn't
    /// [`MessageType::Regular`] or [`MessageType::Reply`] or has role
    /// subscription data, which are edge-cases that can't be replicated
    /// correctly
    ///
    /// Returns [`Error::ContentInvalid`] if the message's content is
    /// invalid, this may happen when the author has used Nitro perks to send a
    /// message with over 2000 characters
    pub fn from_message(message: &'a Message, http: &'a Client) -> Result<Self, Error> {
        if message.activity.is_some() || message.application.is_some() {
            return Err(Error::RichPresence);
        }
        if message.thread.is_some()
            || message.id == message.channel_id.cast()
            || message
                .flags
                .is_some_and(|flags| flags.contains(MessageFlags::HAS_THREAD))
        {
            return Err(Error::Thread);
        }
        if message
            .flags
            .is_some_and(|flags| flags.contains(MessageFlags::IS_VOICE_MESSAGE))
        {
            return Err(Error::Voice);
        }
        if !matches!(message.kind, MessageType::Regular | MessageType::Reply)
            || message.role_subscription_data.is_some()
        {
            return Err(Error::System);
        }
        twilight_validate::message::content(&message.content).map_err(|_| Error::ContentInvalid)?;

        let guild_id = message.guild_id.ok_or(Error::NotInGuild)?;

        let url_components = component::filter_valid(&message.components);
        let has_invalid_components = message.components != url_components;

        Ok(MessageSource {
            source_id: message.id,
            source_channel_id: message.channel_id,
            content: message.content.clone(),
            embeds: &message.embeds,
            tts: message.tts,
            flags: message.flags,
            channel_id: message.channel_id,
            guild_id,
            guild_emoji_ids: None,
            username: message
                .member
                .as_ref()
                .and_then(|member| member.nick.as_ref())
                .unwrap_or(&message.author.name)
                .clone(),
            avatar_url: avatar_url(
                message.author.id,
                guild_id,
                message.author.discriminator,
                message.author.avatar,
                message.member.as_ref().and_then(|member| member.avatar),
            ),
            webhook_name: "Message Cloner".to_owned(),
            sticker_info: sticker::Info {
                exists: !message.sticker_items.is_empty(),
            },
            reaction_info: reaction::Info {
                reactions: &message.reactions,
            },
            attachment_info: attachment::Info {
                attachments: &message.attachments,
                #[cfg(feature = "upload")]
                attachments_upload: vec![],
            },
            component_info: component::Info {
                url_components,
                has_invalid_components,
            },
            thread_info: thread::Info::Unknown,
            webhook: None,
            later_messages: later_messages::Info {
                messages: vec![],
                is_complete: false,
                is_source_created: false,
                is_later_message_sources_created: false,
            },
            response: None,
            http,
        })
    }
}

#[allow(clippy::option_if_let_else)]
fn avatar_url(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    user_discriminator: u16,
    user_avatar: Option<ImageHash>,
    member_avatar: Option<ImageHash>,
) -> String {
    if let Some(avatar) = member_avatar {
        format!(
            "https://cdn.discordapp.com/guilds/{guild_id}/users/{}/avatars/{avatar}.{}",
            user_id,
            if avatar.is_animated() { "gif" } else { "png" }
        )
    } else if let Some(avatar) = user_avatar {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{avatar}.{}",
            user_id,
            if avatar.is_animated() { "gif" } else { "png" }
        )
    } else {
        format!(
            "https://cdn.discordapp.com/embed/avatars/{}.png",
            if user_discriminator == 0 {
                (user_id.get() >> 22_u8) % 6
            } else {
                u64::from(user_discriminator % 5)
            }
        )
    }
}
