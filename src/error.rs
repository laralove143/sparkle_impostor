//! The [`Error`] enum

use twilight_validate::message::MessageValidationError;

/// Error type returned in this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Source message is related to rich presence
    #[error("source message is related to rich presence")]
    SourceRichPresence,
    /// Source message has an attachment
    #[error("source message has an attachment")]
    SourceAttachment,
    /// Source message has a component
    #[error("source message has a component")]
    SourceComponent,
    /// Source message has a reaction
    #[error("source message has a reaction")]
    SourceReaction,
    /// Source message has a sticker
    #[error("source message has a sticker")]
    SourceSticker,
    /// Source message has a thread created from it
    #[error("source message has a thread created from it")]
    SourceThread,
    /// Source message is a voice message
    #[error("source message is a voice message")]
    SourceVoice,
    /// Source message is a system message
    #[error("source message is a system message")]
    SourceSystem,
    /// Source message's content is invalid
    ///
    /// This happens when the author has used Nitro perks to send a message with
    /// over 2000 characters
    #[error("source message's content is invalid")]
    SourceContentInvalid,
    /// Username of the source message's author is invalid
    ///
    /// This happens because usernames or nicks don't have the same requirements
    /// as webhook usernames
    #[error("username of the source message's author is invalid")]
    SourceUsernameInvalid,
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
}
