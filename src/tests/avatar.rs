use crate::{
    clone_message,
    error::Error,
    tests::{Context, IMAGE_BASE64},
};

#[tokio::test]
async fn default() -> Result<(), Error> {
    let ctx = Context::new().await;

    ctx.http.update_current_user().avatar(None).await?;
    let message = ctx
        .create_message()
        .content("default avatar")?
        .await?
        .model()
        .await?;
    clone_message(&message, &ctx.http).await?;

    Ok(())
}

#[tokio::test]
async fn image() -> Result<(), Error> {
    let ctx = Context::new().await;

    ctx.http
        .update_current_user()
        .avatar(Some(IMAGE_BASE64))
        .await?;
    let message = ctx
        .create_message()
        .content("image avatar")?
        .await?
        .model()
        .await?;
    clone_message(&message, &ctx.http).await?;

    Ok(())
}
