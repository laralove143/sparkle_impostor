use crate::common::Context;

mod common;

#[tokio::test]
async fn exists() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let referenced_message = ctx
        .create_message()
        .content("referenced message *(should be replied to in the next message)*")?
        .await?
        .model()
        .await?;

    let mut message = ctx
        .create_message()
        .content(
            "message reference *(should be cloned with an embed containing the referenced \
             message)*",
        )?
        .reply(referenced_message.id)
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .handle_reference()?
        .create()
        .await?;

    Ok(())
}

#[tokio::test]
async fn deleted() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let referenced_message = ctx
        .create_message()
        .content("referenced message deleted *(should be deleted)*")?
        .await?
        .model()
        .await?;

    let mut message = ctx
        .create_message()
        .content(
            "message reference *(should be cloned with an embed saying the referenced message was \
             deleted)*",
        )?
        .reply(referenced_message.id)
        .await?
        .model()
        .await?;

    ctx.http
        .delete_message(ctx.channel_id, referenced_message.id)
        .await?;

    message = ctx
        .http
        .message(ctx.channel_id, message.id)
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .handle_reference()?
        .create()
        .await?;

    Ok(())
}
