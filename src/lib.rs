#![warn(
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::default_numeric_fallback,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::mixed_read_write_in_expression,
    clippy::mod_module_files,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::single_char_lifetime_names,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::try_err,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // nightly lints:
    // fuzzy_provenance_casts,
    // lossy_provenance_casts,
    // must_not_suspend,
    // non_exhaustive_omitted_patterns,
)]
#![allow(clippy::redundant_pub_crate)]
#![doc = include_str!("../README.md")]

use twilight_http::Client;
use twilight_model::{
    channel::{
        message::{Embed, MessageFlags, MessageType},
        Message,
    },
    id::{marker::ChannelMarker, Id},
};

use crate::error::Error;

pub mod error;
#[cfg(test)]
mod tests;

/// A message that can be copied
///
/// Can be mutated to override some fields, for example to clone it to another
/// channel
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MessageSource<'message> {
    /// Content of the message
    pub content: &'message str,
    /// Embeds in the message
    pub embeds: &'message [Embed],
    /// Whether the message has text-to-speech enabled
    pub tts: bool,
    /// Flags of the message
    pub flags: Option<MessageFlags>,
    /// ID of the channel the message is in
    pub channel_id: Id<ChannelMarker>,
    /// Username of the message's author
    pub username: &'message str,
    /// URL of message author's avatar
    pub avatar_url: String,
}

impl<'message> MessageSource<'message> {
    /// Create [`MessageSource`] from a [`Message`]
    ///
    /// # Errors
    ///
    /// Returns [`Error::SourceRichPresence`] if the message is related
    /// to rich presence, which can't be recreated by bots
    ///
    /// Returns [`Error::SourceAttachment`] if the message has an attachment,
    /// this will be handled more gracefully in the future
    ///
    /// Returns [`Error::SourceComponent`] if the message has a component, which
    /// would be broken since the components would then be sent to the cloner
    /// bot
    ///
    /// Returns [`Error::SourceReaction`] if the message has a reaction, this
    /// will be handled more gracefully in the future
    ///
    /// Returns [`Error::SourceSticker`] if the message has a sticker, which
    /// webhook messages can't have
    ///
    /// Returns [`Error::SourceThread`] if the message has a thread created from
    /// it, this will be handled more gracefully in the future
    ///
    /// Returns [`Error::SourceVoice`] if the message is a voice message, which
    /// bots currently can't create
    ///
    /// Returns [`Error::SourceSystem`] if the message's type isn't
    /// [`MessageType::Regular`] or [`MessageType::Reply`] or has role
    /// subscription data, which are edge-cases that can't be replicated
    /// correctly
    ///
    /// Returns [`Error::SourceContentInvalid`] if the message's content is
    /// invalid
    ///
    /// Returns [`Error::SourceUsernameInvalid`] if username of the message's
    /// author is invalid
    pub fn from_message(message: &'message Message) -> Result<Self, Error> {
        if message.activity.is_some() || message.application.is_some() {
            return Err(Error::SourceRichPresence);
        }
        if !message.attachments.is_empty() {
            return Err(Error::SourceAttachment);
        }
        if !message.components.is_empty() {
            return Err(Error::SourceComponent);
        }
        if !message.reactions.is_empty() {
            return Err(Error::SourceReaction);
        }
        if !message.sticker_items.is_empty() {
            return Err(Error::SourceSticker);
        }
        if message.thread.is_some()
            || message
                .flags
                .is_some_and(|flags| flags.contains(MessageFlags::HAS_THREAD))
        {
            return Err(Error::SourceThread);
        }
        if message
            .flags
            .is_some_and(|flags| flags.contains(MessageFlags::IS_VOICE_MESSAGE))
        {
            return Err(Error::SourceVoice);
        }
        if !matches!(message.kind, MessageType::Regular | MessageType::Reply)
            || message.role_subscription_data.is_some()
        {
            return Err(Error::SourceSystem);
        }
        twilight_validate::message::content(&message.content)
            .map_err(|_| Error::SourceContentInvalid)?;
        twilight_validate::request::webhook_username(&message.author.name)
            .map_err(|_| Error::SourceUsernameInvalid)?;

        Ok(Self {
            content: &message.content,
            embeds: &message.embeds,
            tts: message.tts,
            flags: message.flags,
            channel_id: message.channel_id,
            username: message
                .member
                .as_ref()
                .and_then(|member| member.nick.as_ref())
                .unwrap_or(&message.author.name),
            avatar_url: if let (Some(guild_id), Some(avatar)) = (
                message.guild_id,
                message.member.as_ref().and_then(|member| member.avatar),
            ) {
                format!(
                    "https://cdn.discordapp.com/guilds/{guild_id}/users/{}/avatars/{avatar}.{}",
                    message.author.id,
                    if avatar.is_animated() { "gif" } else { "png" }
                )
            } else if let Some(avatar) = message.author.avatar {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{avatar}.{}",
                    message.author.id,
                    if avatar.is_animated() { "gif" } else { "png" }
                )
            } else {
                format!(
                    "https://cdn.discordapp.com/embed/avatars/{}.png",
                    if message.author.discriminator == 0 {
                        (message.author.id.get() >> 22_u8) % 6
                    } else {
                        u64::from(message.author.discriminator % 5)
                    }
                )
            },
        })
    }

    /// Executes a webhook using the given source
    ///
    /// The message's channel is requested from the HTTP API to handle messages
    /// in threads
    ///
    /// Creates a webhook called "Message Cloner" if one made by the bot in the
    /// channel doesn't exist
    ///
    /// Make sure the bot has these required permissions:
    /// - [`Permissions::SEND_TTS_MESSAGES`]
    /// - [`Permissions::MENTION_EVERYONE`]
    /// - [`Permissions::USE_EXTERNAL_EMOJIS`]
    /// - [`Permissions::MANAGE_WEBHOOKS`]
    ///
    /// # Warnings
    ///
    /// If the message has a reply, it will be stripped, since webhook messages
    /// can't have references
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting the channel or webhook, or creating
    /// the webhook fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the channel or
    /// webhook fails
    ///
    /// Returns [`Error::Validation`] if the webhook name is invalid
    ///
    /// Returns [`Error::MessageValidation`] if the given message is invalid,
    /// shouldn't happen unless the message was mutated
    ///
    /// # Panics
    ///
    /// If the webhook that was just created doesn't have a token
    ///
    /// If the message is in a thread and its parent ID is `None`
    pub async fn create(&self, http: &Client) -> Result<(), Error> {
        let channel = http.channel(self.channel_id).await?.model().await?;
        let (channel_id, maybe_thread_id) = if channel.kind.is_thread() {
            (channel.parent_id.unwrap(), Some(channel.id))
        } else {
            (channel.id, None)
        };

        let webhook = if let Some(webhook) = http
            .channel_webhooks(channel_id)
            .await?
            .models()
            .await?
            .into_iter()
            .find(|webhook| webhook.token.is_some())
        {
            webhook
        } else {
            http.create_webhook(channel_id, "Message Cloner")?
                .await?
                .model()
                .await?
        };
        let token = webhook.token.unwrap();

        let mut execute_webhook = http
            .execute_webhook(webhook.id, &token)
            .content(self.content)?
            .username(self.username)?
            .avatar_url(&self.avatar_url)
            .embeds(self.embeds)?
            .tts(self.tts);

        if let Some(thread_id) = maybe_thread_id {
            execute_webhook = execute_webhook.thread_id(thread_id);
        }

        if let Some(flags) = self.flags {
            execute_webhook = execute_webhook.flags(flags);
        }

        execute_webhook.await?;

        Ok(())
    }
}
