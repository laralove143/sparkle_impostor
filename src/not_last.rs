//! Handling the message to clone not being the last one in the channel

use twilight_http::api_error::ApiError;
use twilight_model::channel::Message;

use crate::{error::Error, MessageSource};

impl<'a> MessageSource<'a> {
    /// Check if this is the last message in the channel, return
    /// [`Error::SourceNotLast`] if not
    ///
    /// Make sure the bot has these additional permissions
    /// - [`Permissions::READ_MESSAGE_HISTORY`]
    /// - [`Permissions::VIEW_CHANNEL`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting channel messages fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    ///
    /// Returns [`Error::MissingPermissionReadMessageHistory`] if the bot
    /// doesn't have [`Permissions::READ_MESSAGE_HISTORY`]
    pub async fn check_not_last(self) -> Result<MessageSource<'a>, Error> {
        let messages = self
            .http
            .channel_messages(self.channel_id)
            .limit(1)?
            .await?
            .model()
            .await?;

        let last_message = messages
            .last()
            .ok_or(Error::MissingPermissionReadMessageHistory)?;

        if self.source_id != last_message.id {
            return Err(Error::SourceNotLast);
        }

        Ok(self)
    }

    /// Handle the message not being the last one in the channel by cloning all
    /// messages sent after it
    ///
    /// This costs `messages_sent_after / 100 + messages_sent_after + ratelimit
    /// retries + 1` additional requests, assuming no limit is passed
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
    /// Limit is up to how many messages to resend, returns
    /// [`Error::SourceBeforeLimit`] if above, pass `None` to disable the limit,
    /// this is potentially very expensive
    ///
    /// # Warnings
    ///
    /// This must be called after [`MessageSource::create`] to keep the order
    /// right
    ///
    /// Messages that can't be cloned (See [`MessageSource::from_message`])
    /// aren't resent
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting channel messages or executing the
    /// webhook fails, or if the execution was rate-limited 3 times
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    ///
    /// Returns [`Error::SourceBeforeLimit`] if a limit is given and the message
    /// is before it
    pub async fn create_later_messages(
        self,
        limit: Option<u16>,
    ) -> Result<MessageSource<'a>, Error> {
        let mut messages = vec![];
        loop {
            let message_batch = self
                .http
                .channel_messages(self.source_channel_id)
                .limit(
                    // add 1 for message sent in self.create, 1 to check if above the limit
                    limit
                        .map_or(100, |limit_inner| limit_inner.saturating_add(2))
                        .min(100),
                )?
                .after(
                    messages
                        .last()
                        .map_or(self.source_id, |message: &Message| message.id),
                )
                .await?
                .models()
                .await?
                .into_iter()
                // skip message sent in self.create
                .skip(usize::from(
                    self.channel_id == self.source_channel_id && messages.is_empty(),
                ))
                .rev();

            let is_done = message_batch.len() == 0 || message_batch.len() % 100 != 0;

            messages.extend(message_batch);

            if limit.is_some_and(|limit_inner| messages.len() > usize::from(limit_inner)) {
                return Err(Error::SourceBeforeLimit);
            }

            if is_done {
                break;
            }
        }

        for message_source in messages.iter().filter_map(|message| {
            MessageSource::from_message(message, self.http).map_or(None, |mut source| {
                source.thread_info = self.thread_info;
                source.channel_id = self.channel_id;
                Some(source)
            })
        }) {
            for i in 0..=3_u8 {
                match message_source.clone().create().await {
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
}
