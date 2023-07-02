//! Handling the message to clone not being the last one in the channel

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

    /// Return [`MessageSource`] for messages sent after this
    ///
    /// Make sure the bot has these additional permissions
    /// - [`Permissions::READ_MESSAGE_HISTORY`]
    /// - [`Permissions::VIEW_CHANNEL`]
    ///
    /// # Warnings
    ///
    /// This method is potentially very expensive unless
    /// [`MessageSource::check_is_in_last`] was called
    ///
    /// This must be called after [`MessageSource::create`] to keep the message
    /// order
    ///
    /// If the bot doesn't have [`Permissions::READ_MESSAGE_HISTORY`], it'll
    /// always return an empty vector, since that's what Discord responds with
    ///
    /// # Errors
    ///
    /// The vector element will be an error if the message can't be resent (See
    /// [`MessageSource::from_message`])
    ///
    /// Returns [`Error::Http`] if getting channel messages fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    pub async fn later_messages(
        &'a mut self,
    ) -> Result<Vec<Result<MessageSource<'a>, Error>>, Error> {
        self.set_later_messages(None).await?;

        Ok(self
            .later_messages
            .messages
            .iter()
            .map(|message| {
                MessageSource::from_message(message, self.http).map(|mut source| {
                    source.thread_info = self.thread_info;
                    source.channel_id = self.channel_id;
                    source
                })
            })
            .collect())
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
