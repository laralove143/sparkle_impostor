//! The [`Error`] enum

use twilight_validate::{channel::ChannelValidationError, message::MessageValidationError};

/// Error type returned in this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Message is not in a guild
    #[error("message is not in a guild")]
    NotInGuild,
    /// Message is related to rich presence
    #[error("message is related to rich presence")]
    RichPresence,
    /// Message is a voice message
    #[error("message is a voice message")]
    Voice,
    /// Message is a system message
    #[error("message is a system message")]
    System,
    /// Message has a non-URL component
    #[error("message has a non-URL component")]
    Component,
    /// Message has a reaction
    #[error("message has a reaction")]
    Reaction,
    /// Message has more reaction emojis than the limit
    #[error("message has more reaction emojis than {0}")]
    ReactionAboveLimit(u8),
    /// Message has a reaction emoji with count higher than 1
    #[error("message has a reaction emoji with count higher than one")]
    ReactionCountMultiple,
    /// Message has a reaction emoji that's not unicode
    #[error("message has a reaction emoji that's not unicode")]
    ReactionCustom,
    /// Message has an external reaction emoji
    #[error("message has an external reaction emoji")]
    ReactionExternal,
    /// Message has a sticker
    #[error("message has a sticker")]
    Sticker,
    /// Sticker in message can't be linked to
    #[error("sticker in message can't be linked to")]
    StickerLinkInvalid,
    /// Message has an attachment
    #[error("message has an attachment")]
    Attachment,
    /// Message's attachments are too large
    ///
    /// This happens when the author has used Nitro perks to send a message with
    /// over 25 MB of attachments
    #[cfg(feature = "upload")]
    #[error("message's attachments are too large")]
    AttachmentTooLarge,
    /// Message's content is invalid
    #[error("message's content is invalid")]
    ContentInvalid,
    /// Message is not in last `n` messages
    #[error("message is not in last {0} messages")]
    SourceAboveLimit(u16),
    /// Message has not been created yet
    #[error("message has not been created yet")]
    NotCreated,
    /// Deleting messages would use more than `n` requests
    #[error("deleting messages would use more than {0} request")]
    DeleteRequestCountAboveLimit(u16),
    /// A Twilight HTTP error occurred
    #[error("{0}")]
    Http(#[from] twilight_http::Error),
    /// A deserialize body error was returned
    #[error("{0}")]
    DeserializeBody(#[from] twilight_http::response::DeserializeBodyError),
    /// A validation error was returned
    #[error("{0}")]
    Validation(#[from] twilight_validate::request::ValidationError),
    /// A message validation error was returned
    #[error("{0}")]
    MessageValidation(#[from] MessageValidationError),
    /// A channel validation error was returned
    #[error("{0}")]
    ChannelValidation(#[from] ChannelValidationError),
    /// A reqwest error was returned
    #[cfg(feature = "upload")]
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}
