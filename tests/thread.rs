use common::Context;
use twilight_model::channel::ChannelType;

mod common;

#[tokio::test]
async fn message() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_thread(
            ctx.channel_id,
            "sparkle impostor thread",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?;

    let mut message = ctx
        .http
        .create_message(thread.id)
        .content("message in thread")?
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .handle_thread()
        .await?
        .create()
        .await?
        .handle_thread_created()
        .await?;

    Ok(())
}

#[tokio::test]
async fn ignore() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("thread ignore not in thread *(should be cloned as normal)*")?
        .await?
        .model()
        .await?;

    ctx.clone_message(&mut message).await?;

    Ok(())
}

#[tokio::test]
async fn create() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread_create_message = ctx
        .create_message()
        .content("thread created *(should be cloned with a new thread)*")?
        .await?
        .model()
        .await?;

    ctx.http
        .create_thread_from_message(
            thread_create_message.channel_id,
            thread_create_message.id,
            "sparkle impostor thread create",
        )?
        .await?;

    let mut message = ctx
        .http
        .message(thread_create_message.channel_id, thread_create_message.id)
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .handle_thread()
        .await?
        .create()
        .await?
        .handle_thread_created()
        .await?;

    Ok(())
}

#[tokio::test]
async fn forum_post() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "sparkle impostor forum post")
        .message()
        .content("forum post *(should be cloned as another post)*")?
        .await?
        .model()
        .await?
        .message;

    ctx.message_source(&mut message)?
        .handle_thread()
        .await?
        .create()
        .await?;

    Ok(())
}

#[tokio::test]
async fn forum_message() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "sparkle impostor forum message")
        .message()
        .content("forum first message *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    let mut message = ctx
        .http
        .create_message(thread.channel.id)
        .content("forum message")?
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .handle_thread()
        .await?
        .create()
        .await?;

    Ok(())
}
