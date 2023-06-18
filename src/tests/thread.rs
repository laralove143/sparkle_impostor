use twilight_model::channel::ChannelType;

use crate::{clone_message, tests::Context};

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

    clone_message(&message, &ctx.http).await?;

    Ok(())
}