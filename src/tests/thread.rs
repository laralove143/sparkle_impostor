use twilight_model::channel::ChannelType;

use crate::{error::Error, tests::Context, MessageSourceBuilder};

#[tokio::test]
async fn thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_thread(
            ctx.channel_id,
            "impostor test thread",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?;

    let message = ctx
        .http
        .create_message(thread.id)
        .content("thread")?
        .await?
        .model()
        .await?;

    ctx.clone_message(&message).await?;

    Ok(())
}

#[tokio::test]
async fn ignore_thread_message_in_thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let thread = ctx
        .http
        .create_thread(
            ctx.channel_id,
            "impostor ignore test thread",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?;

    let message = ctx
        .http
        .create_message(thread.id)
        .content("thread *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    assert!(matches!(
        MessageSourceBuilder::new()
            .ignore_threads()
            .build_from_message(&message)?
            .create(&ctx.http)
            .await,
        Err(Error::Http(_))
    ));

    Ok(())
}

#[tokio::test]
async fn ignore_thread_message_not_in_thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("ignore thread *(should be cloned as normal)*")?
        .await?
        .model()
        .await?;

    MessageSourceBuilder::new()
        .ignore_threads()
        .build_from_message(&message)?
        .create(&ctx.http)
        .await?;

    Ok(())
}
