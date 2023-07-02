//! Handling the message to clone not being the last one in the channel

use twilight_http::api_error::ApiError;
use twilight_model::channel::Message;

use crate::{error::Error, MessageSource};

/// Info about the later messages in the channel
#[derive(Debug, Clone)]
pub struct Info {
    /// Messages sent later
    ///
    /// Has to be an owned value because some methods set it from owned values
    pub messages: Vec<Message>,
    /// Whether there's no more messages
    pub is_complete: bool,
    /// Whether [`MessageSource::create`] was called
    pub is_source_created: bool,
}

impl<'a> MessageSource<'a> {
    /// Check if this is in the last `n` messages in the channel, return
    /// [`Error::SourceNotIn`] if not
    ///
    /// Make sure the bot has these additional permissions
    /// - [`Permissions::READ_MESSAGE_HISTORY`]
    /// - [`Permissions::VIEW_CHANNEL`]
    ///
    /// # Warnings
    ///
    /// If [`MessageSource::create`] was called before this method, don't
    /// account for that message when setting the limit
    ///
    /// If the bot doesn't have [`Permissions::READ_MESSAGE_HISTORY`], it'll act
    /// as if this is the last message, since that's what Discord responds with
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceNotIn`] if the message isn't in the last `n`
    /// messages
    ///
    /// Returns [`Error::Http`] if getting channel messages fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    pub async fn check_is_in_last(mut self, n: u16) -> Result<MessageSource<'a>, Error> {
        self.set_later_messages(Some(n)).await?;

        Ok(self)
    }

    /// Handle the message not being the last one in the channel by cloning all
    /// messages sent after it
    ///
    /// This costs `messages_sent_after / 100 + messages_sent_after +
    /// ratelimit_retries` additional requests, making this method
    /// potentially very expensive if [`MessageSource::check_is_in_last`]
    /// wasn't called, [`MessageSource::check_is_in_last`] also saves these
    /// messages so that they're not requested again
    ///
    /// Because rate-limits for webhook executions can't be handled beforehand,
    /// retries each execution up to 3 times, if all of these are rate-limited,
    /// returns the HTTP error
    ///
    /// Make sure the bot has these additional permissions
    /// - [`Permissions::READ_MESSAGE_HISTORY`]
    /// - [`Permissions::VIEW_CHANNEL`]
    ///
    /// The documentation of [`MessageSource::create`] applies here as well
    ///
    /// # Warnings
    ///
    /// This must be called after [`MessageSource::create`] to keep the message
    /// order
    ///
    /// Messages that can't be cloned (See [`MessageSource::from_message`])
    /// aren't resent
    ///
    /// If the bot doesn't have [`Permissions::READ_MESSAGE_HISTORY`], it'll act
    /// as if this is the last message, since that's what Discord responds with
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting channel messages or executing the
    /// webhook fails, or if the execution was rate-limited 3 times
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    pub async fn create_later_messages(mut self) -> Result<MessageSource<'a>, Error> {
        self.set_later_messages(None).await?;

        for message in &self.later_messages.messages {
            for i in 0..=3_u8 {
                let Ok(mut message_source) =
                    MessageSource::from_message(message, self.http) else {
                    break;
                };
                message_source.thread_info = self.thread_info;
                message_source.channel_id = self.channel_id;

                match message_source.create().await {
                    Ok(_) => break,
                    Err(Error::Http(err))
                        if matches!(
                            err.kind(),
                            twilight_http::error::ErrorType::Response {
                                error: ApiError::Ratelimited(_),
                                ..
                            }
                        ) =>
                    {
                        if i == 3 {
                            return Err(Error::Http(err));
                        }
                        continue;
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(self)
    }

    async fn set_later_messages(&mut self, limit: Option<u16>) -> Result<(), Error> {
        loop {
            if let Some(limit_inner) = limit {
                if self.later_messages.messages.len() >= usize::from(limit_inner) {
                    return Err(Error::SourceNotIn(limit_inner));
                }
            }
            if self.later_messages.is_complete {
                return Ok(());
            }

            let message_batch = self
                .http
                .channel_messages(self.source_channel_id)
                .limit(limit.unwrap_or(100).min(100))?
                .after(
                    self.later_messages
                        .messages
                        .last()
                        .map_or(self.source_id, |message: &Message| message.id),
                )
                .await?
                .models()
                .await?;

            self.later_messages.is_complete =
                message_batch.is_empty() || message_batch.len() % 100 != 0;

            self.later_messages.messages.extend(
                message_batch
                    .into_iter()
                    // skip message sent in self.create
                    .skip(usize::from(
                        self.later_messages.is_source_created
                            && self.channel_id == self.source_channel_id
                            && self.later_messages.messages.is_empty(),
                    ))
                    .rev(),
            );
        }
    }
}
