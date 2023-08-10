# ✨🥸 Sparkle Impostor

A third party crate for [Twilight](https://github.com/twilight-rs/twilight) to execute Discord webhooks that clone an
existing message

## ✨ Features

> This has grown to be a more complicated _(and painful)_ project than expected, so let me list what it does

Opt-in features for edge-cases:

- Clone attachments or stickers by linking to them or re-uploading them
- Clone URL components
- Clone messages sent after the original message
- Clone reactions
- Clone references by putting an embed
- Clone messages in a thread/forum post or messages used to start a thread/forum post
- Sanitize invalid usernames
- Delete the original message and messages sent after

General features:

- Replicate the author's user or member avatar, embeds, anything possible
- Handle rate-limit retries
- Builder-pattern to keep your code clean
- Avoid clones and unnecessary deserialization
- Widely tested with integration tests _(Almost 1:1 LOC for source and tests)_

## 📦 Cargo Features

- `upload`: Enables methods for re-uploading attachments

## 🙏 Feedback

Although widely tested, there may still be bugs, or you might have feature suggestions, please create issues for these!

## 🧪 Testing

The crate uses integration tests as opposed to unit tests to test real-world usage. It creates a message and clones it,
then the tester checks if the message is cloned as expected

Before starting, set these environment variables, you can also put them in a `.env` file:

- `BOT_TOKEN`: The token of the bot to use for testing
- `CHANNEL_ID`: The channel in which the messages and webhooks will be crated
- `FORUM_CHANNEL_ID`: The forum channel in which cloning messages/threads in forum channels will be tested
- `NOT_LAST_SOURCE_THREAD_ID`: The bot will create a thread and spam to 200 in it the first time the tests are ran, to
  avoid doing this again, set this to the ID of this thread
- `GUILD_EMOJI_ID`: ID of an emoji that's in the guild `CHANNEL_ID` is in

Required permissions in `CHANNEL_ID` and `FORUM_CHANNEL_ID`:

- `VIEW_CHANNEL`
- `MANAGE_WEBHOOKS`
- `SEND_MESSAGES`

Required additional permissions in `FORUM_CHANNEL_ID`:

- `CREATE_POSTS`
- `SEND_MESSAGES_IN_POSTS`

Test with a single thread to avoid race conditions: `cargo test --all-features -- --test-threads=1`