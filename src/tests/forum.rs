use crate::{error::Error, tests::Context};

#[tokio::test]
async fn create() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "impostor forum thread")
        .message()
        .content("forum first message *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    assert!(matches!(
        ctx.clone_message(&thread.message).await,
        Err(Error::SourceThread)
    ));

    Ok(())
}

#[tokio::test]
async fn message() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_forum_thread(ctx.forum_channel_id, "impostor forum thread")
        .message()
        .content("forum first message *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    let message = ctx
        .http
        .create_message(thread.channel.id)
        .content("forum message")?
        .await?
        .model()
        .await?;

    ctx.clone_message(&message).await?;

    Ok(())
}
