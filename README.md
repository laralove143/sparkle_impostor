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

Test with a single thread to avoid race conditions: `cargo test -- --test-threads=1`

The bot's username and avatar will be changed as part of the testing

Some things can't be tested due to Discord limitations:

- **GIF avatar:** Bots can't have a GIF avatar
- **Guild avatar:** Bots can't have a guild avatar
- **Rich presence messages:** Bots can't send rich presence messages
- **Voice messages:** Bots can't send voice messages