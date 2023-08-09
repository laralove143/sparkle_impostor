//! Handling the message to clone being in a thread

use twilight_model::{
    channel::{Channel, ChannelType},
    id::{marker::ChannelMarker, Id},
};

use crate::{error::Error, MessageSource};

/// Thread [`MessageSource`] is in
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Info {
    /// Message is in a thread, but the thread hasn't been created from the
    /// message
    ///
    /// Wrapped value is the thread's ID
    In(Id<ChannelMarker>),
    /// A non-post thread has been created from the message
    ///
    /// Wrapped value is the thread
    Created(Box<Channel>),
    /// A post in a forum channel has been created from the message
    ///
    /// Wrapped value is the thread
    CreatedPost(Box<Channel>),
    /// A thread has been created from the message, but it's not known whether
    /// it's a forum channel post or not
    ///
    /// Wrapped value is the thread
    CreatedUnknown(Box<Channel>),
    /// The message is not in a thread
    NotIn,
    /// Thread info hasn't been checked
    Unknown,
}

impl Info {
    pub(crate) fn id(&self) -> Option<Id<ChannelMarker>> {
        match self {
            Self::In(id) => Some(*id),
            Self::Created(thread) | Self::CreatedPost(thread) | Self::CreatedUnknown(thread) => {
                Some(thread.id)
            }
            _ => None,
        }
    }
}

impl<'a> MessageSource<'a> {
    /// Handle the message being in a thread
    ///
    /// This requires getting the channel with another HTTP request
    ///
    /// # Warnings
    ///
    /// Must be called before [`MessageSource::create`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting the channel fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the channel fails
    #[allow(clippy::missing_panics_doc)]
    pub async fn handle_thread(mut self) -> Result<MessageSource<'a>, Error> {
        if !matches!(self.thread_info, Info::Unknown | Info::CreatedUnknown(_)) {
            return Ok(self);
        }

        let thread = if let Info::CreatedUnknown(thread) = self.thread_info {
            thread
        } else {
            Box::new(self.http.channel(self.channel_id).await?.model().await?)
        };

        if !thread.kind.is_thread() {
            self.thread_info = Info::NotIn;
            return Ok(self);
        }

        self.channel_id = thread.parent_id.unwrap();

        self.thread_info = if self.source_id == thread.id.cast() {
            let channel = self.http.channel(self.channel_id).await?.model().await?;

            if channel.kind == ChannelType::GuildForum {
                Info::CreatedPost(thread)
            } else {
                Info::Created(thread)
            }
        } else {
            Info::In(thread.id)
        };

        self.source_thread_id = self.thread_info.id();

        Ok(self)
    }

    /// Handle a thread being created from the message
    ///
    /// # Errors
    ///
    /// Returns [`Error::ChannelValidation`] if the thread is invalid, shouldn't
    /// happen unless the it was mutated
    ///
    /// Returns [`Error::Http`] if creating the thread fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the thread fails
    ///
    /// # Panics
    ///
    /// If called before [`MessageSource::create`]
    pub async fn handle_thread_created(mut self) -> Result<MessageSource<'a>, Error> {
        if let Info::Created(thread) = &self.thread_info {
            let thread_new = self
                .http
                .create_thread_from_message(
                    self.channel_id,
                    self.response.as_mut().unwrap().model().await?.id,
                    thread.name.as_ref().unwrap(),
                )?
                .await?
                .model()
                .await?;

            self.thread_info = Info::Created(Box::new(thread_new));
        }

        Ok(self)
    }
}
