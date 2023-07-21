use twilight_model::channel::Message;

use crate::common::Context;

mod common;

#[tokio::test]
async fn one() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = create_message(&ctx).await?;

    ctx.message_source(&mut message)?.delete().await?;

    Ok(())
}

#[tokio::test]
async fn bulk() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = create_message(&ctx).await?;
    create_message(&ctx).await?;

    let mut message_source = ctx.message_source(&mut message)?;
    message_source.later_messages().await?;
    message_source.delete().await?;

    Ok(())
}

#[tokio::test]
async fn check_in_last_side_effect() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = create_message(&ctx).await?;
    create_message(&ctx).await?;

    ctx.message_source(&mut message)?
        .check_is_in_last(2)
        .await?
        .delete()
        .await?;

    Ok(())
}

async fn create_message(ctx: &Context) -> Result<Message, anyhow::Error> {
    Ok(ctx
        .create_message()
        .content("delete *(should be deleted.. hopefully you're not reading this)*")?
        .await?
        .model()
        .await?)
}
