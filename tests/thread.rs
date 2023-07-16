use common::Context;
use sparkle_impostor::error::Error;
use twilight_model::channel::ChannelType;

mod common;

#[tokio::test]
async fn thread() -> Result<(), anyhow::Error> {
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

    ctx.message_source(
        &ctx.http
            .create_message(thread.id)
            .content("thread")?
            .await?
            .model()
            .await?,
    )?
    .handle_thread()
    .await?
    .create()
    .await?;

    Ok(())
}

#[tokio::test]
async fn ignore_in_thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    assert!(matches!(
        ctx.clone_message(
            &ctx.http
                .create_message(
                    ctx.http
                        .create_thread(
                            ctx.channel_id,
                            "sparkle impostor thread ignore",
                            ChannelType::PublicThread,
                        )?
                        .await?
                        .model()
                        .await?
                        .id
                )
                .content("thread ignore message in thread *(should not be cloned)*")?
                .await?
                .model()
                .await?
        )
        .await,
        Err(Error::Http(_))
    ));

    Ok(())
}

#[tokio::test]
async fn ignore_not_in_thread() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    ctx.clone_message(
        &ctx.create_message()
            .content("thread ignore not in thread *(should be cloned as normal)*")?
            .await?
            .model()
            .await?,
    )
    .await?;

    Ok(())
}
