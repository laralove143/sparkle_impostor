//! Handling the message to clone not being the last one in the channel

use twilight_model::channel::Message;
#[cfg(doc)]
use twilight_model::guild::Permissions;
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

use crate::{error::Error, MessageSource};

/// Info about the later messages in the channel
#[derive(Debug, Clone)]
pub struct Info {
    /// Messages sent later
    ///
    /// Has to be an owned value because some methods set it from owned values
    pub messages: Vec<Message>,
    /// Whether there are no more messages
    pub is_complete: bool,
    /// Whether [`MessageSource::create`] was called
    pub is_source_created: bool,
    /// Whether [`MessageSource::later_messages`] or
    /// [`MessageSource::later_messages_batched`] was called
    pub is_later_message_sources_created: bool,
}

impl<'a> MessageSource<'a> {
    /// Check if this is in the last `n` messages in the channel, return
    /// [`Error::SourceAboveLimit`] if not
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
    /// Returns [`Error::SourceAboveLimit`] if the message isn't in the last `n`
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
    /// If the bot doesn't have [`Permissions::READ_MESSAGE_HISTORY`], it'll
    /// always return an empty vector, since that's what Discord responds with
    ///
    /// Should not be combined with [`MessageSource::later_messages_batched`]
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
    pub async fn later_messages(&mut self) -> Result<Vec<Result<MessageSource<'_>, Error>>, Error> {
        self.set_later_messages(None).await?;

        Ok(self.later_message_sources())
    }

    /// Return [`MessageSource`] for messages sent after this after combining
    /// messages from the same author to the same message
    ///
    /// This combines the messages' content separated with a newline, it's
    /// provided to reduce the number of webhook executions
    ///
    /// See [`MessageSource::later_messages`] for more
    ///
    /// # Warnings
    ///
    /// Should not be combined with [`MessageSource::later_messages`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting channel messages fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing channel messages
    /// fails
    pub async fn later_messages_batched(
        &mut self,
    ) -> Result<Vec<Result<MessageSource<'_>, Error>>, Error> {
        self.set_later_messages(None).await?;

        // clone to another vec because removing elements from the vec is more expensive
        let mut messages_batched = vec![];

        for message in self.later_messages.messages.clone() {
            let Some(last_message) = messages_batched.last_mut() else {
                messages_batched.push(message);
                continue;
            };

            if last_message.author.id == message.author.id
                && last_message
                    .content
                    .chars()
                    .count()
                    .saturating_add(message.content.chars().count())
                    < MESSAGE_CONTENT_LENGTH_MAX
            // not <= because we push '\n' too
            {
                last_message.content.push('\n');
                last_message.content.push_str(&message.content);
            } else {
                messages_batched.push(message);
            }
        }
        self.later_messages.messages = messages_batched;

        Ok(self.later_message_sources())
    }

    async fn set_later_messages(&mut self, limit: Option<u16>) -> Result<(), Error> {
        loop {
            if let Some(limit_inner) = limit {
                if self.later_messages.messages.len() >= usize::from(limit_inner) {
                    return Err(Error::SourceAboveLimit(limit_inner));
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

    fn later_message_sources(&mut self) -> Vec<Result<MessageSource<'_>, Error>> {
        self.later_messages.is_later_message_sources_created = true;

        self.later_messages
            .messages
            .iter()
            .map(|message| {
                MessageSource::from_message(message, self.http).map(|mut source| {
                    source.thread_info = self.thread_info;
                    source.channel_id = self.channel_id;
                    source
                })
            })
            .collect()
    }
}
