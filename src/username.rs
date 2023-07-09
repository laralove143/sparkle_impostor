use twilight_validate::request::{
    ValidationErrorType, WEBHOOK_USERNAME_LIMIT_MAX, WEBHOOK_USERNAME_LIMIT_MIN,
};

use crate::MessageSource;

impl MessageSource<'_> {
    /// Sanitize the username if it's invalid
    ///
    /// This is necessary because usernames or nicks don't have the same
    /// requirements as webhook usernames
    ///
    /// If the username is under [`WEBHOOK_USERNAME_LIMIT_MIN`], appends
    /// `append` to it, make sure `append` is under 32 characters and is not
    /// empty
    ///
    /// If the username is over [`WEBHOOK_USERNAME_LIMIT_MAX`], trims it and
    /// ends it with "..."
    ///
    /// Replaces invalid substrings with `replace`, make sure it's over
    /// [`WEBHOOK_USERNAME_LIMIT_MIN`] characters and under 6 characters
    #[must_use]
    pub fn sanitize_username(mut self, append: &str, replace: &str) -> Self {
        if let Err(ValidationErrorType::WebhookUsername { len, substring }) = self.check_username()
        {
            if let Some(len_inner) = len {
                if len_inner < WEBHOOK_USERNAME_LIMIT_MIN {
                    self.username.push_str(append);
                }
                if len_inner > WEBHOOK_USERNAME_LIMIT_MAX {
                    self.username = self
                        .username
                        .chars()
                        .take(WEBHOOK_USERNAME_LIMIT_MAX - 3)
                        .collect();
                    self.username.push_str("...");
                }
            }

            if let Some(substring_inner) = substring {
                self.username = self.username.replace(substring_inner, replace);
            }
        }

        self
    }

    /// Check that the author's username is valid
    ///
    /// You can use this method when [`MessageSource::sanitize_username`] isn't
    /// enough
    ///
    /// # Errors
    ///
    /// [`ValidationErrorType::WebhookUsername`] if the username is invalid
    pub fn check_username(&self) -> Result<(), ValidationErrorType> {
        twilight_validate::request::webhook_username(&self.username)
            .map_err(|err| err.into_parts().0)?;
        Ok(())
    }
}
