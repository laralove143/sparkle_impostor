use common::Context;
use sparkle_impostor::error::Error;
use twilight_model::{
    channel::{ChannelType, Message},
    id::Id,
};
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

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
        Err(Error::SourceNotIn(2))
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
async fn create_later() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut messages: Vec<Message> = vec![];
    loop {
        let get_messages = ctx
            .http
            .channel_messages(ctx.not_last_source_thread_id)
            .limit(100)?;

        let mut message_batch = if messages.is_empty() {
            get_messages.await?.models().await?
        } else {
            get_messages
                .before(messages.last().unwrap().id)
                .await?
                .models()
                .await?
        };

        let is_done = message_batch.is_empty() || message_batch.len() % 100 != 0;

        messages.append(&mut message_batch);

        if is_done {
            break;
        }
    }

    let mut message_source = ctx.message_source(messages.last_mut().unwrap())?;
    message_source.channel_id = ctx
        .http
        .create_thread(
            ctx.channel_id,
            "sparkle impostor create later messages target",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?
        .id;

    message_source = message_source.handle_thread().await?.create().await?;

    let later_messages = message_source.later_messages().await?;

    assert!(later_messages.iter().all(Result::is_ok));
    for later_message in later_messages {
        later_message?.create().await?;
    }

    Ok(())
}

#[tokio::test]
async fn batched() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut messages = vec![];
    for i in 1_u8..=4 {
        messages.push(
            ctx.create_message()
                .content(&format!(
                    "batched messages {i} *(should be cloned with 2 and 3 combined into one \
                     message)*"
                ))?
                .await?
                .model()
                .await?,
        );
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

    let mut message_source = ctx.message_source(&mut message)?.create().await?;

    let later_messages = message_source.later_messages_batched().await?;

    assert!(later_messages.iter().all(Result::is_ok));
    for later_message in later_messages {
        later_message?.create().await?;
    }

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

    let mut message_source = ctx.message_source(&mut message)?.create().await?;

    let later_messages = message_source.later_messages_batched().await?;

    assert!(later_messages.iter().all(Result::is_ok));
    for later_message in later_messages {
        later_message?.create().await?;
    }

    Ok(())
}
