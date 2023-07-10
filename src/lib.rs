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

use twilight_http::{request::channel::webhook::ExecuteWebhookAndWait, Client};
#[cfg(doc)]
use twilight_model::guild::Permissions;
use twilight_model::{
    channel::message::{Embed, MessageFlags},
    id::{
        marker::{ChannelMarker, MessageMarker, WebhookMarker},
        Id,
    },
};

use crate::error::Error;

mod constructor;
pub mod error;
pub mod not_last;
#[cfg(test)]
mod tests;
mod thread;
mod username;

/// A message that can be cloned
///
/// Can be mutated to override some fields, for example to clone it to another
/// channel, but fields starting with `source` shouldn't be mutated
///
/// You can also provide some of the fields, for example from your cache, so
/// that they won't be received over the HTTP API
///
/// # Warning
///
/// Many of the fields here are stateful, there are no guarantees on the
/// validity of these since this doesn't have access to the gateway, this means
/// you should use and drop this struct as fast as you can
#[derive(Debug)]
pub struct MessageSource<'a> {
    /// Message's ID
    pub source_id: Id<MessageMarker>,
    /// ID of the channel the source message is in
    pub source_channel_id: Id<ChannelMarker>,
    /// Content of the message
    pub content: &'a str,
    /// Embeds in the message
    pub embeds: &'a [Embed],
    /// Whether the message has text-to-speech enabled
    pub tts: bool,
    /// Flags of the message
    pub flags: Option<MessageFlags>,
    /// ID of the channel the message is in
    ///
    /// If the message is in a thread, this should be the parent thread's ID
    pub channel_id: Id<ChannelMarker>,
    /// Username of the message's author
    pub username: String,
    /// URL of message author's avatar
    pub avatar_url: String,
    /// Name to be used for the webhook that will be used to create the message
    pub webhook_name: String,
    /// Info about the message's thread
    pub thread_info: thread::Info,
    /// Webhook ID and token to execute to clone messages with
    pub webhook: Option<(Id<WebhookMarker>, String)>,
    /// Messages sent after the source
    pub later_messages: not_last::Info,
    /// The client to use for requests
    pub http: &'a Client,
}

impl<'a> MessageSource<'a> {
    /// Executes a webhook using the given source
    ///
    /// If a webhook called the set name or *Message Cloner* in the channel
    /// doesn't exist, creates it
    ///
    /// Make sure the bot has these required permissions:
    /// - [`Permissions::SEND_TTS_MESSAGES`]
    /// - [`Permissions::MENTION_EVERYONE`]
    /// - [`Permissions::USE_EXTERNAL_EMOJIS`]
    /// - [`Permissions::MANAGE_WEBHOOKS`]
    ///
    /// Because rate-limits for webhook executions can't be handled
    /// beforehand, retries each execution up to 3 times, if all of these
    /// are rate-limited, returns the HTTP error
    ///
    /// # Warnings
    ///
    /// Other methods on [`MessageSource`] are provided to handle edge-cases,
    /// not calling them before this may make this method fail
    ///
    /// If the message has a reply, it will be stripped, since webhook messages
    /// can't have replies
    ///
    /// If calling this on the same webhook repeatedly, it's rate-limited on
    /// every try after the 50th execution in tests
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if getting, creating or executing the webhook
    /// fails
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the webhook fails
    ///
    /// Returns [`Error::Validation`] if the webhook name is invalid
    ///
    /// Returns [`Error::MessageValidation`] if the given message is invalid,
    /// shouldn't happen unless the message was mutated
    ///
    /// # Panics
    ///
    /// If the webhook that was just created doesn't have a token
    pub async fn create(mut self) -> Result<MessageSource<'a>, Error> {
        self.set_webhook().await?;

        for i in 0..=3_u8 {
            match self.webhook_exec()?.await {
                Ok(_) => break,
                Err(err)
                    if matches!(
                        err.kind(),
                        twilight_http::error::ErrorType::Response {
                            error: twilight_http::api_error::ApiError::Ratelimited(_),
                            ..
                        }
                    ) =>
                {
                    if i == 3 {
                        return Err(Error::Http(err));
                    }
                    continue;
                }
                Err(err) => return Err(Error::Http(err)),
            }
        }

        self.later_messages.is_source_created = true;

        Ok(self)
    }

    /// Set the name of the webhook to use for creating messages
    ///
    /// Defaults to *Message Cloner* if not called
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn webhook_name(mut self, name: String) -> Self {
        self.webhook_name = name;
        self
    }

    async fn set_webhook(&mut self) -> Result<(), Error> {
        if self.webhook.is_some() {
            return Ok(());
        }

        let webhook = if let Some(webhook) = self
            .http
            .channel_webhooks(self.channel_id)
            .await?
            .models()
            .await?
            .into_iter()
            .find(|webhook| {
                webhook.token.is_some() && webhook.name.as_ref() == Some(&self.webhook_name)
            }) {
            webhook
        } else {
            self.http
                .create_webhook(self.channel_id, &self.webhook_name)?
                .await?
                .model()
                .await?
        };
        self.webhook = Some((webhook.id, webhook.token.unwrap()));

        Ok(())
    }

    fn webhook_exec(&self) -> Result<ExecuteWebhookAndWait<'_>, Error> {
        let (webhook_id, webhook_token) = self.webhook.as_ref().unwrap();

        let mut execute_webhook = self
            .http
            .execute_webhook(*webhook_id, webhook_token)
            .content(self.content)?
            .username(&self.username)?
            .avatar_url(&self.avatar_url)
            .embeds(self.embeds)?
            .tts(self.tts);

        if let thread::Info::Known(Some(thread_id)) = self.thread_info {
            execute_webhook = execute_webhook.thread_id(thread_id);
        }

        if let Some(flags) = self.flags {
            execute_webhook = execute_webhook.flags(flags);
        }

        // not waiting causes race condition issues in the client
        Ok(execute_webhook.wait())
    }
}
