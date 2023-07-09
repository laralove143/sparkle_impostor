use twilight_validate::request::WEBHOOK_USERNAME_LIMIT_MAX;

use crate::{tests::Context, MessageSource};

#[tokio::test]
async fn sanitize_too_short() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("sanitize username too short *(should be cloned with username 'aa')*")?
        .await?
        .model()
        .await?;
    message.author.name = "a".to_owned();

    MessageSource::from_message(&message, &ctx.http)?
        .sanitize_username("a", "")
        .create()
        .await?;

    Ok(())
}

#[tokio::test]
async fn sanitize_too_long() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content(
            "sanitize username too long *(should be cloned with username of 29 'a' ending with \
             '...')*",
        )?
        .await?
        .model()
        .await?;
    message.author.name = "a".repeat(WEBHOOK_USERNAME_LIMIT_MAX + 1);

    MessageSource::from_message(&message, &ctx.http)?
        .sanitize_username("", "")
        .create()
        .await?;

    Ok(())
}

#[tokio::test]
async fn sanitize_substring() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("sanitize username invalid substring *(should be cloned with username 'aa')*")?
        .await?
        .model()
        .await?;
    message.author.name = "clyde".to_owned();

    MessageSource::from_message(&message, &ctx.http)?
        .sanitize_username("", "aa")
        .create()
        .await?;

    Ok(())
}
