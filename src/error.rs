//! The [`Error`] enum

use twilight_validate::{channel::ChannelValidationError, message::MessageValidationError};

/// Error type returned in this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Source message is not in a guild
    #[error("source message is not in a guild")]
    NotInGuild,
    /// Source message is related to rich presence
    #[error("source message is related to rich presence")]
    RichPresence,
    /// Source message is a voice message
    #[error("source message is a voice message")]
    Voice,
    /// Source message is a system message
    #[error("source message is a system message")]
    System,
    /// Source message has a non-URL component
    #[error("source message has a non-URL component")]
    Component,
    /// Source message has a reaction
    #[error("source message has a reaction")]
    Reaction,
    /// Source message has more reaction emojis than the limit
    #[error("source message has more reaction emojis than {0}")]
    ReactionAboveLimit(u8),
    /// Source message has a reaction emoji with count higher than 1
    #[error("source message has a reaction emoji with count higher than one")]
    ReactionCountMultiple,
    /// Source message has a reaction emoji that's not unicode
    #[error("source message has a reaction emoji that's not unicode")]
    ReactionCustom,
    /// Source message has an external reaction emoji
    #[error("source message has an external reaction emoji")]
    ReactionExternal,
    /// Source message has a sticker
    #[error("source message has a sticker")]
    Sticker,
    /// Sticker in message can't be linked to
    #[error("sticker in message can't be linked to")]
    StickerLinkInvalid,
    /// Source message has an attachment
    #[error("source message has an attachment")]
    Attachment,
    /// Source message's attachments are too large
    ///
    /// This happens when the author has used Nitro perks to send a message with
    /// over 25 MB of attachments
    #[cfg(feature = "upload")]
    #[error("source message's attachments are too large")]
    AttachmentTooLarge,
    /// Source message's content is invalid
    #[error("source message's content is invalid")]
    ContentInvalid,
    /// Source message is not in last `n` messages
    #[error("source message is not in last {0} messages")]
    SourceAboveLimit(u16),
    /// Source message has not been created yet
    #[error("source message has not been created yet")]
    NotCreated,
    /// Deleting messages would use more than `n` requests
    #[error("deleting messages would use more than {0} request")]
    DeleteRequestCountAboveLimit(u16),
    /// An Twilight HTTP error occurred
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
