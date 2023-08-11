//! Handling the message having reactions

use twilight_http::request::channel::reaction::RequestReactionType;
#[cfg(doc)]
use twilight_model::guild::Permissions;
use twilight_model::{
    channel::message::{Reaction, ReactionType},
    id::{marker::EmojiMarker, Id},
};

use crate::{error::Error, MessageSource};

/// Defines what to allow when checking reactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckBehavior {
    /// Allow no reactions
    None,
    /// Allow up to the given number of reaction emojis
    ///
    /// This is useful to limit the number of requests sent in recreating the
    /// reactions
    ///
    /// Currently a message is allowed to have up to 20 reaction emojis
    Limit(u8),
    /// Only allow reactions if there is only one of each emoji
    ///
    /// This is useful because multiple reactions can't be recreated, since the
    /// bot can't react with the same emoji twice
    CountOne,
    /// Only allow unicode emojis, not Nitro emojis
    ///
    /// Not using this will make [`MessageSource::handle_reaction`]
    /// request the guild to check if the emoji is from the current guild and
    /// filter other ones
    Unicode,
    /// Only allow unicode emojis or emojis in the current guild
    ///
    /// This is useful when you don't want to filter external emojis
    NotExternal,
}

/// Info about reactions in [`MessageSource`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info<'a> {
    /// Reactions in the message
    pub reactions: &'a [Reaction],
}

impl<'a> MessageSource<'a> {
    /// Check that the message has no reactions
    ///
    /// You can call the same function repeatedly with different
    /// [`CheckBehavior`]s
    ///
    /// This function is only async if [`CheckBehavior::NotExternal`] was passed
    ///
    /// # Warnings
    ///
    /// Super reactions currently can't be received, so they're ignored
    ///
    /// # Errors
    ///
    /// Returns [`Error::Reaction`] if [`CheckBehavior::None`] was passed
    /// and the message has a reaction
    ///
    /// Returns [`Error::ReactionAboveLimit`] if [`CheckBehavior::Limit`]
    /// was passed and the message has more reactions than the limit
    ///
    /// Returns [`Error::ReactionCountMultiple`] if
    /// [`CheckBehavior::CountOne`] was passed and the message has a reaction
    /// emoji with count higher than 1
    ///
    /// Returns [`Error::ReactionCustom`] if
    /// [`CheckBehavior::Unicode`] was passed and the message has a non-unicode
    /// reaction emoji
    ///
    /// Returns [`Error::ReactionExternal`] if
    /// [`CheckBehavior::NotExternal`] was passed and and the message has an
    /// external reaction emoji
    ///
    /// Returns [`Error::Http`] if [`CheckBehavior::NotExternal`] was passed and
    /// getting guild emojis fails
    ///
    /// Returns [`Error::DeserializeBody`] if [`CheckBehavior::NotExternal`] was
    /// passed and deserializing guild emojis fails
    #[allow(clippy::missing_panics_doc)]
    pub async fn check_reaction(&mut self, behavior: CheckBehavior) -> Result<(), Error> {
        match behavior {
            CheckBehavior::None if !self.reaction_info.reactions.is_empty() => Err(Error::Reaction),
            CheckBehavior::Limit(limit)
                if self.reaction_info.reactions.len() <= usize::from(limit) =>
            {
                Err(Error::ReactionAboveLimit(limit))
            }
            CheckBehavior::CountOne
                if self
                    .reaction_info
                    .reactions
                    .iter()
                    .any(|reaction| reaction.count > 1) =>
            {
                Err(Error::ReactionCountMultiple)
            }
            CheckBehavior::Unicode if custom_emoji_exists(self.reaction_info.reactions) => {
                Err(Error::ReactionCustom)
            }
            CheckBehavior::NotExternal => {
                if !custom_emoji_exists(self.reaction_info.reactions) {
                    return Ok(());
                }

                self.set_guild_emojis().await?;

                if self.reaction_info.reactions.iter().any(|reaction| {
                    is_reaction_emoji_external(reaction, self.guild_emoji_ids.as_ref().unwrap())
                }) {
                    return Err(Error::ReactionExternal);
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Re-create the reactions
    ///
    /// The reaction authors and counts will naturally be lost, their author
    /// will be the bot and counts will be 1
    ///
    /// Guild emojis will have to be requested if any reaction emoji isn't
    /// unicode to check if the emoji is external
    ///
    /// Make sure the bot has these additional permissions:
    /// - [`Permissions::ADD_REACTIONS`]
    /// - [`Permissions::READ_MESSAGE_HISTORY`]
    ///
    /// # Warnings
    ///
    /// Super reactions currently can't be received, so they're ignored
    ///
    /// Using this without [`MessageSource::check_reaction`] may cause loss in
    /// reactions
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotCreated`] if [`MessageSource::create`] wasn't called
    /// yet
    ///
    /// Returns [`Error::Http`] if getting guild emojis fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the message or guild
    /// emojis failed
    #[allow(clippy::missing_panics_doc)]
    pub async fn handle_reaction(mut self) -> Result<MessageSource<'a>, Error> {
        let reactions = if custom_emoji_exists(self.reaction_info.reactions) {
            self.set_guild_emojis().await?;

            self.reaction_info
                .reactions
                .iter()
                .filter(|reaction| {
                    !is_reaction_emoji_external(reaction, self.guild_emoji_ids.as_ref().unwrap())
                })
                .collect::<Vec<_>>()
        } else {
            self.reaction_info.reactions.iter().collect()
        };

        if reactions.is_empty() {
            return Ok(self);
        }

        let message_id = self
            .response
            .as_mut()
            .ok_or(Error::NotCreated)?
            .model()
            .await?
            .id;

        for reaction in reactions {
            let request_reaction = match &reaction.emoji {
                ReactionType::Custom { id, name, .. } => RequestReactionType::Custom {
                    id: *id,
                    name: name.as_deref(),
                },
                ReactionType::Unicode { name } => RequestReactionType::Unicode { name },
            };

            self.http
                .create_reaction(self.channel_id, message_id, &request_reaction)
                .await?;
        }

        Ok(self)
    }
}

fn custom_emoji_exists(reactions: &[Reaction]) -> bool {
    reactions
        .iter()
        .any(|reaction| matches!(reaction.emoji, ReactionType::Custom { .. }))
}

fn is_reaction_emoji_external(reaction: &Reaction, guild_emoji_ids: &[Id<EmojiMarker>]) -> bool {
    let ReactionType::Custom { id, .. } = reaction.emoji else {
        return false;
    };

    !guild_emoji_ids.contains(&id)
}
