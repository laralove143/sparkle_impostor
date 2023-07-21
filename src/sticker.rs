//! Handling the message having stickers

use crate::{error::Error, MessageSource};

/// Info about the message's stickers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    /// Whether the message has any stickers
    pub exists: bool,
}

impl MessageSource<'_> {
    /// Check that the message has no stickers
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sticker`] if the message has a sticker
    #[allow(clippy::missing_const_for_fn)]
    pub fn check_sticker(self) -> Result<Self, Error> {
        if self.sticker_info.exists {
            return Err(Error::Sticker);
        }

        Ok(self)
    }
}
