use std::time::{Duration, SystemTime};

#[cfg(docs)]
use twilight_model::guild::Permissions;
use twilight_model::id::{marker::MessageMarker, Id};
use twilight_util::snowflake::Snowflake;

use crate::{error::Error, MessageSource};

struct MessagesDelete {
    bulk: Vec<Vec<Id<MessageMarker>>>,
    single: Vec<Id<MessageMarker>>,
}

impl<'a> MessageSource<'a> {
    /// Check if [`MessageSource::delete`] would use more than `n` requests
    ///
    /// If [`MessageSource::later_messages`] or
    /// [`MessageSource::later_messages_batched`] wasn't called, `n` is always 1
    ///
    /// Each message older than 2 weeks uses 1 request, others use
    /// `other_message_count` divided by
    /// [`twilight_validate::channel::CHANNEL_BULK_DELETE_MESSAGES_MAX`] rounded
    /// up requests
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeleteRequestCountAboveLimit`] if
    /// [`MessageSource::delete`] would use more than `n` requests
    pub fn check_delete_request_count_in(&self, n: u16) -> Result<(), Error> {
        let messages_delete = self.messages_delete();

        if messages_delete
            .single
            .len()
            .saturating_add(messages_delete.bulk.len())
            > usize::from(n)
        {
            return Err(Error::DeleteRequestCountAboveLimit(n));
        }

        Ok(())
    }

    /// Delete the original message
    ///
    /// If [`MessageSource::later_messages`] or
    /// [`MessageSource::later_messages_batched`] was called, later messages
    /// will also be deleted
    ///
    /// If there is a message older than two weeks, they'll be
    /// deleted individually since bulk delete isn't valid for these messages,
    /// see [`MessageSource::check_delete_request_count_in`] if this is not the expected
    /// behavior
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
        let messages_delete = self.messages_delete();

        for message_ids_chunk in messages_delete.bulk {
            self.http
                .delete_messages(self.source_channel_id, &message_ids_chunk)
                .unwrap()
                .await?;
        }

        for message_id in messages_delete.single {
            self.http
                .delete_message(self.source_channel_id, message_id)
                .await?;
        }

        Ok(self)
    }

    fn messages_delete(&self) -> MessagesDelete {
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

        MessagesDelete {
            bulk: message_ids_bulk_delete
                .chunks(twilight_validate::channel::CHANNEL_BULK_DELETE_MESSAGES_MAX)
                .map(ToOwned::to_owned)
                .collect(),
            single: message_ids_delete,
        }
    }
}
