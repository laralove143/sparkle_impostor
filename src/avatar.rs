//! Handling the message's avatar

use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    util::ImageHash,
};

/// Info about the avatar of [`MessageSource`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Info {
    /// Avatar's URL
    ///
    /// `None` if it hasn't been set yet
    pub url: Option<String>,
    /// Avatar owner's user ID
    pub user_id: Id<UserMarker>,
    /// Member avatar owner's guild ID
    pub guild_id: Id<GuildMarker>,
    /// Avatar owner's discriminator
    pub user_discriminator: u16,
    /// Avatar owner's global avatar hash
    ///
    /// `None` if they don't have a global avatar
    pub user_avatar: Option<ImageHash>,
    /// Avatar owner's guild avatar hash
    ///
    /// `None` if they don't have a guild avatar
    pub member_avatar: Option<ImageHash>,
}

impl Info {
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn set_url(&mut self) {
        let url = if let Some(avatar) = self.member_avatar {
            format!(
                "https://cdn.discordapp.com/guilds/{}/users/{}/avatars/{avatar}.{}",
                self.guild_id,
                self.user_id,
                hash_extension(avatar)
            )
        } else if let Some(avatar) = self.user_avatar {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{avatar}.{}",
                self.user_id,
                hash_extension(avatar)
            )
        } else {
            format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png",
                if self.user_discriminator == 0 {
                    (self.user_id.get() >> 22_u8) % 6
                } else {
                    u64::from(self.user_discriminator % 5)
                }
            )
        };

        self.url = Some(url);
    }
}

const fn hash_extension(hash: ImageHash) -> &'static str {
    if hash.is_animated() {
        "gif"
    } else {
        "png"
    }
}
