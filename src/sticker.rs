use crate::{error::Error, MessageSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub exists: bool,
}

impl MessageSource<'_> {
    /// Check that the message has no stickers
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceSticker`] if the message has a sticker
    #[allow(clippy::missing_const_for_fn)]
    pub fn check_sticker(self) -> Result<Self, Error> {
        if self.sticker_info.exists {
            return Err(Error::SourceSticker);
        }

        Ok(self)
    }
}
