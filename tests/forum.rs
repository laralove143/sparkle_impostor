use sparkle_impostor::error::Error;

use crate::common::Context;

mod common;

#[tokio::test]
async fn create() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "sparkle impostor forum thread")
        .message()
        .content("forum first message *(should not be cloned)*")?
        .await?
        .model()
        .await?
        .message;

    assert!(matches!(
        ctx.clone_message(&mut message).await,
        Err(Error::Thread)
    ));

    Ok(())
}

#[tokio::test]
async fn message() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "sparkle impostor forum thread")
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
