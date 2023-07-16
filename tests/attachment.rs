use common::Context;
use sparkle_impostor::error::Error;
use twilight_model::http::attachment::Attachment;
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;

mod common;

#[tokio::test]
async fn link() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("attachment link *(should be cloned with links at the bottom)*")?
        .attachments(&[
            Attachment {
                description: None,
                file: vec![1],
                filename: "attachment_issues.txt".to_owned(),
                id: 0,
            },
            Attachment {
                description: None,
                file: vec![1],
                filename: "attachment_issues.txt".to_owned(),
                id: 1,
            },
        ])?
        .await?
        .model()
        .await?;

    ctx.message_source(&message)?
        .handle_attachment_link()?
        .create()
        .await?;

    Ok(())
}

#[tokio::test]
async fn link_content_too_long() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("attachment link content too long *(should not be cloned)*")?
        .attachments(&[Attachment {
            description: None,
            file: vec![1],
            filename: "real_attachment_issues.txt".to_owned(),
            id: 0,
        }])?
        .await?
        .model()
        .await?;

    message.content.push_str(&"a".repeat(
        MESSAGE_CONTENT_LENGTH_MAX
            - message.content.chars().count()
            - message.attachments.first().unwrap().url.chars().count()
            - 2,
    ));

    ctx.message_source(&message)?.handle_attachment_link()?;

    message.content.push('a');
    assert!(matches!(
        ctx.message_source(&message)?.handle_attachment_link(),
        Err(Error::SourceContentInvalid)
    ));

    Ok(())
}

#[cfg(feature = "upload")]
#[tokio::test]
async fn upload() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("attachment upload *(should be cloned with the attachment)*")?
        .attachments(&[Attachment {
            description: None,
            file: vec![1],
            filename: "copyright_reserved.txt".to_owned(),
            id: 0,
        }])?
        .await?
        .model()
        .await?;

    ctx.message_source(&message)?
        .handle_attachment_upload()
        .await?
        .create()
        .await?;

    Ok(())
}
