use std::time::{Duration, SystemTime};

#[cfg(docs)]
use twilight_model::guild::Permissions;
use twilight_util::snowflake::Snowflake;

use crate::{error::Error, MessageSource};

impl<'a> MessageSource<'a> {
    /// Delete the original message
    ///
    /// If [`MessageSource::later_messages`] or
    /// [`MessageSource::later_messages_batched`] was called, later messages
    /// will also be deleted, for messages older than two weeks, they'll be
    /// deleted individually since bulk delete isn't valid for these messages
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if deleting the messages fails
    ///
    /// # Panics
    ///
    /// If two weeks ago or the time a message was sent can't be represented
    /// with [`SystemTime`] on the current platform
    pub async fn delete(self) -> Result<MessageSource<'a>, Error> {
        let mut message_ids = vec![self.source_id];

        if self.later_messages.is_later_message_sources_created {
            message_ids.extend(
                self.later_messages
                    .messages
                    .iter()
                    .map(|message| message.id),
            );
        }

        let two_weeks_ago = SystemTime::now()
            .checked_sub(Duration::from_secs(60 * 60 * 24 * 7 * 2))
            .unwrap();

        let (mut message_ids_bulk_delete, mut message_ids_delete) = message_ids
            .into_iter()
            .partition::<Vec<_>, _>(|message_id| {
                SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_millis(
                        message_id.timestamp().try_into().unwrap(),
                    ))
                    .unwrap()
                    >= two_weeks_ago
            });
        if message_ids_bulk_delete.len() == 1 {
            message_ids_delete.push(message_ids_bulk_delete.pop().unwrap());
        }

        for message_ids_chunk in message_ids_bulk_delete
            .chunks(twilight_validate::channel::CHANNEL_BULK_DELETE_MESSAGES_MAX)
        {
            self.http
                .delete_messages(self.source_channel_id, message_ids_chunk)
                .unwrap()
                .await?;
        }

        for message_id in message_ids_delete {
            self.http
                .delete_message(self.source_channel_id, message_id)
                .await?;
        }

        Ok(self)
    }
}
