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
    clippy::get_unwrap,
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
    elided_lifetimes_in_paths
    explicit_outlives_requirements,
    fuzzy-provenance-casts,
    keyword_idents,
    let_underscore_drop,
    lossy_provenance_casts,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    must_not_suspend,
    non_ascii_idents,
    non_exhaustive_omitted_patterns,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax
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
    unused_results,
    unused_tuple_struct_fields,
    variant_size_differences,
)]
#![doc = include_str!("../README.md")]

use sparkle_convenience::reply::Reply;
use twilight_model::{
    id::{
        marker::{ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
    util::ImageHash,
};

/// The message to clone, also the entrypoint of this library
///
/// Any of the fields, including the wrapped [`Reply`], can be modified to
/// change the executed message
#[derive(Debug)]
pub struct MessageSource {
    /// The ID of the channel the message is in
    pub channel_id: Id<ChannelMarker>,
    /// The ID of the guild the message is in, if it's in a guild
    pub guild_id: Option<Id<GuildMarker>>,
    /// The ID of the message's author
    pub author_id: Id<UserMarker>,
    /// The global avatar of the message's author, if there is one
    pub user_avatar: Option<ImageHash>,
    /// The guild avatar of the message's author, if there is one
    pub member_avatar: Option<ImageHash>,
    /// The other properties of the message
    pub reply: Reply,
}
