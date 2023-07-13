#[cfg(feature = "reqwest")]
use reqwest::Client;
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

use crate::{error::Error, MessageSource};

/// Info about attachments in [`MessageSource`]
#[derive(Clone, Debug, PartialEq)]
pub struct Info<'a> {
    /// Attachments in the message
    pub attachments: &'a [twilight_model::channel::Attachment],
    /// Attachments to re-upload
    #[cfg(feature = "upload")]
    pub attachments_upload: Vec<twilight_model::http::attachment::Attachment>,
}

impl MessageSource<'_> {
    /// Check that the message has no attachments
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceAttachment`] if the message has an attachment
    #[allow(clippy::missing_const_for_fn)]
    pub fn check_attachment(self) -> Result<Self, Error> {
        if !self.attachment_info.attachments.is_empty() {
            return Err(Error::SourceAttachment);
        }

        Ok(self)
    }

    /// Append links to the attachments into the message content
    ///
    /// If the attachment is an image, it's embedded in the client
    ///
    /// # Warnings
    ///
    /// The link will die after the source message is deleted
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceContentInvalid`] if the message content becomes
    /// too long after adding the links
    pub fn handle_attachment_link(mut self) -> Result<Self, Error> {
        if self.content.chars().count().saturating_add(
            self.attachment_info
                .attachments
                .iter()
                // add 1 for newlines
                .map(|attachment| attachment.url.chars().count().saturating_add(1))
                .reduce(usize::saturating_add)
                .unwrap_or(0),
        ).saturating_add(1) // add 1 for empty line between
            > MESSAGE_CONTENT_LENGTH_MAX
        {
            return Err(Error::SourceContentInvalid);
        }

        self.content.push('\n');
        for attachment in self.attachment_info.attachments {
            self.content.push('\n');
            self.content.push_str(&attachment.url);
        }

        Ok(self)
    }
}

#[cfg(feature = "upload")]
impl<'a> MessageSource<'a> {
    /// Re-upload the attachments
    ///
    /// This downloads and saves the attachments, they're later uploaded in
    /// [`MessageSource::create`]
    ///
    /// # Warnings
    ///
    /// This is an expensive operation since it means downloading and uploading
    /// up to 25 MBs
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceAttachmentTooLarge`] if the combined size of the
    /// attachments is over 25 MB
    ///
    /// Returns [`Error::Reqwest`] if downloading the attachments fails
    pub async fn handle_attachment_upload(mut self) -> Result<MessageSource<'a>, Error> {
        if self
            .attachment_info
            .attachments
            .iter()
            .map(|attachment| attachment.size)
            .sum::<u64>()
            > 25 * 1024 * 1024
        {
            return Err(Error::SourceAttachmentTooLarge);
        }

        let client = Client::new();
        for attachment in self.attachment_info.attachments {
            self.attachment_info.attachments_upload.push(
                twilight_model::http::attachment::Attachment {
                    description: attachment.description.clone(),
                    file: client
                        .get(&attachment.url)
                        .send()
                        .await?
                        .bytes()
                        .await?
                        .to_vec(),
                    filename: attachment.filename.clone(),
                    id: attachment.id.get(),
                },
            );
        }

        Ok(self)
    }
}
