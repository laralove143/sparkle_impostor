use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
    },
    http::attachment::Attachment,
};

use crate::{clone_message, error::Error, tests::Context};

fn message_content(name: &str) -> String {
    format!("{name} *(should not be cloned)*")
}

#[tokio::test]
async fn attachment() -> Result<(), Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content(&message_content("attachment"))?
        .attachments(&[Attachment::from_bytes(
            "attachment.txt".to_owned(),
            b"attachment content".to_vec(),
            1,
        )])?
        .await?
        .model()
        .await?;

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceAttachment)
    ));

    Ok(())
}

#[tokio::test]
async fn component() -> Result<(), Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content(&message_content("component"))?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some("idrathernot".to_owned()),
                disabled: false,
                emoji: None,
                label: Some("red".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            })],
        })])?
        .await?
        .model()
        .await?;

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceComponent)
    ));

    Ok(())
}

#[tokio::test]
async fn reaction() -> Result<(), Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content(&message_content("reaction"))?
        .await?
        .model()
        .await?;

    ctx.http
        .create_reaction(
            message.channel_id,
            message.id,
            &RequestReactionType::Unicode { name: "🥸" },
        )
        .await?;

    message = ctx
        .http
        .message(message.channel_id, message.id)
        .await?
        .model()
        .await?;

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceReaction)
    ));

    Ok(())
}

#[tokio::test]
async fn sticker() -> Result<(), Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content(&message_content("sticker"))?
        .sticker_ids(&[ctx
            .http
            .guild_stickers(ctx.guild_id)
            .await?
            .models()
            .await?
            .get(0)
            .unwrap()
            .id])?
        .await?
        .model()
        .await?;

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceSticker)
    ));

    Ok(())
}

#[tokio::test]
async fn thread_created() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content(&message_content("thread created"))?
        .await?
        .model()
        .await?;

    ctx.http
        .create_thread_from_message(message.channel_id, message.id, "thread")?
        .await?;

    message = ctx
        .http
        .message(message.channel_id, message.id)
        .await?
        .model()
        .await?;

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceThread)
    ));

    Ok(())
}

#[tokio::test]
async fn system() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let pin_message = ctx
        .create_message()
        .content(&message_content("system"))?
        .await?
        .model()
        .await?;

    ctx.http
        .create_pin(pin_message.channel_id, pin_message.id)
        .await?;

    let message = ctx
        .http
        .channel_messages(pin_message.channel_id)
        .limit(1)?
        .await?
        .models()
        .await?
        .remove(0);

    assert!(matches!(
        clone_message(&message, &ctx.http).await,
        Err(Error::SourceSystem)
    ));

    Ok(())
}
