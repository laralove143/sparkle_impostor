# ✨🥸 Sparkle Impostor

A third party crate for [Twilight](https://github.com/twilight-rs/twilight) to execute Discord webhooks that clone an
existing message

## 🤷 Why?

My bots use it, it's open source because why not

## 🧪 Testing

The crate uses integration tests as opposed to unit tests to test real-world usage. It creates a message and clones it,
then the tester checks if the message is cloned as expected

Before starting, set these environment variables, you can also put them in a `.env` file:

- `BOT_TOKEN`: The token of the bot to use for testing
- `CHANNEL_ID`: The channel in which the messages and webhooks will be crated

Required permissions in the given `CHANNEL_ID`:

- `VIEW_CHANNEL`
- `MANAGE_WEBHOOKS`
- `SEND_MESSAGES`

Test with a single thread to avoid race conditions: `cargo test -- --test-threads=1`

Some things can't be tested due to Discord limitations:

- **Rich presence messages:** Bots can't send rich presence messages
- **Nitro content length:** Bots can't send messages that are over 2000 characters long
- **Voice messages:** Bots can't send voice messages