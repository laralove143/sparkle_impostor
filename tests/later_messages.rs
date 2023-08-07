use common::Context;
use sparkle_impostor::error::Error;
use twilight_model::id::Id;
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

use crate::common::create_later_messages;

mod common;

#[tokio::test]
async fn check_err() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("check is in last err *(nothing should be cloned)*")?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;

    assert!(matches!(
        ctx.message_source(&mut message)?.check_is_in_last(2).await,
        Err(Error::SourceAboveLimit(2))
    ));

    Ok(())
}

#[tokio::test]
async fn check_ok() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("check is in ok *(nothing should be cloned)*")?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;

    ctx.message_source(&mut message)?
        .check_is_in_last(3)
        .await?;

    Ok(())
}

#[tokio::test]
async fn create_later_thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut thread_message = ctx
        .http
        .message(ctx.channel_id, ctx.not_last_source_thread_id.cast())
        .await?
        .model()
        .await?;

    let message_source = ctx
        .message_source(&mut thread_message)?
        .handle_thread()
        .await?
        .create()
        .await?
        .handle_thread_created()
        .await?;

    create_later_messages(message_source).await?;

    Ok(())
}

#[tokio::test]
async fn create_later_channel() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut maybe_first_message = None;
    for i in 1..=3 {
        let resp = ctx
            .create_message()
            .content(&format!(
                "channel message create later {i} *(should be cloned with the same order)*"
            ))?
            .await?;

        if i == 1 {
            maybe_first_message = Some(resp.model().await?);
        }
    }
    let mut first_message = maybe_first_message.unwrap();

    let message_source = ctx.message_source(&mut first_message)?.create().await?;

    create_later_messages(message_source).await?;

    Ok(())
}

#[tokio::test]
async fn batched() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut messages = vec![];
    for i in 1_u8..=4 {
        let mut message = ctx
            .create_message()
            .content(&format!(
                "batched messages {i} *(should be cloned with 2 and 3 combined into one message)*"
            ))?
            .await?
            .model()
            .await?;

        message.guild_id = Some(ctx.guild_id);

        messages.push(message);
    }
    messages.get_mut(1).unwrap().author.id = Id::new(1);
    messages.get_mut(2).unwrap().author.id = Id::new(1);

    let mut first_message = messages.remove(0);
    let mut message_source = ctx.message_source(&mut first_message)?.create().await?;

    message_source.later_messages.messages = messages;
    message_source.later_messages.is_complete = true;
    message_source.later_messages.is_source_created = true;

    let later_messages = message_source.later_messages_batched().await?;

    assert!(later_messages.iter().all(Result::is_ok));
    for later_message in later_messages {
        later_message?.create().await?;
    }

    Ok(())
}

#[tokio::test]
async fn batched_content_too_long() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("batched messages content too long *(each message should be cloned separately)*")?
        .await?
        .model()
        .await?;

    for _ in 0_usize..2 {
        ctx.create_message()
            .content(&"a".repeat(MESSAGE_CONTENT_LENGTH_MAX.div_euclid(2)))?
            .await?
            .model()
            .await?;
    }

    let message_source = ctx.message_source(&mut message)?.create().await?;

    create_later_messages(message_source).await?;

    Ok(())
}

#[tokio::test]
async fn batched_content_not_too_long() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("batched messages content not too long *(last two messages should be combined)*")?
        .await?
        .model()
        .await?;

    for i in 0_usize..2 {
        ctx.create_message()
            .content(&"a".repeat(MESSAGE_CONTENT_LENGTH_MAX.div_euclid(2) - i))?
            .await?
            .model()
            .await?;
    }

    let message_source = ctx.message_source(&mut message)?.create().await?;

    create_later_messages(message_source).await?;

    Ok(())
}
