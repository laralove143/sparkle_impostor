use twilight_http::Client;
use twilight_model::id::{marker::ChannelMarker, Id};

use crate::{error::Error, MessageSource};

/// Info about the thread [`MessageSource`] is in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Info {
    /// The thread has been checked
    ///
    /// `None` if the message isn't in a thread, the thread's ID if it is
    Known(Option<Id<ChannelMarker>>),
    /// Thread info hasn't been checked
    Unknown,
}

impl<'a> MessageSource<'a> {
    /// Handle the message being in a thread
    ///
    /// This requires getting the channel with another HTTP request
    ///
    /// # Warnings
    ///
    /// Unless called, trying to clone a message that's in a thread will result
    /// in [`Error::Http`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting the channel fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the channel fails
    ///
    /// # Panics
    ///
    /// If the thread's parent ID is `None`
    pub async fn handle_thread(mut self, http: &Client) -> Result<MessageSource<'a>, Error> {
        if self.thread_info == Info::Unknown {
            let channel = http.channel(self.channel_id).await?.model().await?;
            if channel.kind.is_thread() {
                self.thread_info = Info::Known(Some(channel.id));
                self.channel_id = channel.parent_id.unwrap();
            };
        }

        Ok(self)
    }
}
