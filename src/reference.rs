//! Handling the message having a reference

use twilight_model::channel::{
    message::{embed::EmbedAuthor, Embed},
    Message,
};

use crate::{avatar, error::Error, MessageSource};

/// Info about the message's reference
#[derive(Debug, Clone, PartialEq)]
pub enum Info<'a> {
    /// Message does not have a reference or it hasn't been checked
    None,
    /// Message has a reference, but it's been deleted or is unknown due to a
    /// Discord error
    UnknownOrDeleted,
    /// Message has a known reference
    Reference(&'a Message),
}

impl MessageSource<'_> {
    /// Handle the message having a reference
    ///
    /// This adds an embed with the reference message's info to the message
    ///
    /// # Warnings
    ///
    /// Must be called before [`MessageSource::create`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::MessageValidation`] if the message already has
    /// [`EMBED_COUNT_LIMIT`] embeds
    ///
    /// [`EMBED_COUNT_LIMIT`]: twilight_validate::message::EMBED_COUNT_LIMIT
    #[allow(clippy::missing_panics_doc)]
    pub fn handle_reference(mut self) -> Result<Self, Error> {
        if Info::None == self.reference_info {
            return Ok(self);
        }

        let base_embed = Embed {
            title: Some("Reply to:".to_owned()),
            author: None,
            color: None,
            description: None,
            fields: vec![],
            footer: None,
            image: None,
            kind: String::new(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            url: None,
            video: None,
        };

        // can't create the error manually
        self.embeds.push(base_embed);
        let mut embed = if let Err(err) = twilight_validate::message::embeds(&self.embeds) {
            self.embeds.pop();
            return Err(err.into());
        } else {
            self.embeds.pop().unwrap()
        };

        embed.description = Some(match self.reference_info {
            Info::UnknownOrDeleted => "Unknown or deleted message".to_owned(),
            Info::Reference(message) => message
                .content
                .chars()
                .take(97)
                .chain(['.'; 3])
                .collect::<String>(),
            Info::None => unreachable!(),
        });

        if let Info::Reference(message) = self.reference_info {
            let mut avatar_info = avatar::Info {
                url: None,
                user_id: message.author.id,
                guild_id: self.guild_id,
                user_discriminator: message.author.discriminator,
                user_avatar: message.author.avatar,
                member_avatar: message.member.as_ref().and_then(|member| member.avatar),
            };
            avatar_info.set_url();

            embed.author = Some(EmbedAuthor {
                icon_url: avatar_info.url,
                name: message
                    .member
                    .as_ref()
                    .and_then(|member| member.nick.as_ref())
                    .unwrap_or(&message.author.name)
                    .clone(),
                proxy_icon_url: None,
                url: None,
            });

            embed.url = Some(format!(
                "https://discord.com/channels/{}/{}/{}",
                self.guild_id, message.channel_id, message.id
            ));
        }

        self.embeds.push(embed);

        Ok(self)
    }
}
