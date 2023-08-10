//! Handling the message to clone having attachments or stickers

#[cfg(feature = "reqwest")]
use reqwest::Client;
use twilight_model::channel::message::sticker::{MessageSticker, StickerFormatType};
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

use crate::{error::Error, MessageSource};

/// Info about attachments and stickers in [`MessageSource`]
#[derive(Clone, Debug, PartialEq)]
pub struct Info<'a> {
    /// Stickers in the message
    pub stickers: &'a [MessageSticker],
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
    /// Returns [`Error::Attachment`] if the message has an attachment
    #[allow(clippy::missing_const_for_fn)]
    pub fn check_attachment(self) -> Result<Self, Error> {
        if !self.attachment_sticker_info.attachments.is_empty() {
            return Err(Error::Attachment);
        }

        Ok(self)
    }

    /// Check that the message has no stickers
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sticker`] if the message has a sticker
    #[allow(clippy::missing_const_for_fn)]
    pub fn check_sticker(self) -> Result<Self, Error> {
        if !self.attachment_sticker_info.stickers.is_empty() {
            return Err(Error::Sticker);
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
    /// Returns [`Error::ContentInvalid`] if the message content becomes
    /// too long after adding the links
    pub fn handle_attachment_link(mut self) -> Result<Self, Error> {
        self.append_urls(
            self.attachment_sticker_info
                .attachments
                .iter()
                .map(|attachment| attachment.url.as_str()),
        )?;

        Ok(self)
    }

    /// Append links to the stickers into the message content
    ///
    /// The stickers are embedded in the client, but [`StickerFormatType::Apng`]
    /// stickers aren't animated
    ///
    /// # Errors
    ///
    /// Returns [`Error::StickerLinkInvalid`] if a sticker's
    /// [`StickerFormatType`] is [`StickerFormatType::Lottie`] or
    /// [`StickerFormatType::Unknown`]
    ///
    /// Returns [`Error::ContentInvalid`] if the message content becomes
    /// too long after adding the links
    pub fn handle_sticker_link(mut self) -> Result<Self, Error> {
        let mut sticker_urls = vec![];
        for sticker in self.attachment_sticker_info.stickers {
            sticker_urls.push(format!(
                "https://cdn.discordapp.com/stickers/{}.{}",
                sticker.id,
                match sticker.format_type {
                    StickerFormatType::Gif => "gif",
                    StickerFormatType::Png | StickerFormatType::Apng => "png",
                    _ => return Err(Error::StickerLinkInvalid),
                }
            ));
        }

        self.append_urls(sticker_urls.iter().map(String::as_str))?;

        Ok(self)
    }

    #[allow(single_use_lifetimes)]
    fn append_urls<'a>(
        &mut self,
        urls: impl Iterator<Item = &'a str> + Clone,
    ) -> Result<(), Error> {
        if self
            .content
            .chars()
            .count()
            .saturating_add(
                urls.clone()
                    // add 1 for newlines
                    .map(|url| url.chars().count().saturating_add(1))
                    .reduce(usize::saturating_add)
                    .unwrap_or(0),
            )
            // add 1 for empty line between
            .saturating_add(1)
            > MESSAGE_CONTENT_LENGTH_MAX
        {
            return Err(Error::ContentInvalid);
        }

        self.content.push('\n');
        for url in urls {
            self.content.push('\n');
            self.content.push_str(url);
        }

        Ok(())
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
    /// Returns [`Error::AttachmentTooLarge`] if the combined size of the
    /// attachments is over 25 MB
    ///
    /// Returns [`Error::Reqwest`] if downloading the attachments fails
    pub async fn handle_attachment_upload(mut self) -> Result<MessageSource<'a>, Error> {
        if self
            .attachment_sticker_info
            .attachments
            .iter()
            .map(|attachment| attachment.size)
            .sum::<u64>()
            > 25 * 1024 * 1024
        {
            return Err(Error::AttachmentTooLarge);
        }

        let client = Client::new();
        for attachment in self.attachment_sticker_info.attachments {
            self.attachment_sticker_info.attachments_upload.push(
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
