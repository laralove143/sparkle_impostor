use sparkle_impostor::error::Error;
use twilight_model::channel::message::{
    component::{ActionRow, Button, ButtonStyle},
    Component,
};

use crate::common::Context;

mod common;

#[tokio::test]
async fn url() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("component url *(should be cloned with only the url component)*")?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![invalid_component(), valid_component()],
        })])?
        .await?
        .model()
        .await?;

    ctx.clone_message(&mut message).await?;

    Ok(())
}

#[tokio::test]
async fn check_invalid() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("component error *(should not be cloned)*")?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![invalid_component()],
        })])?
        .await?
        .model()
        .await?;

    assert!(matches!(
        ctx.message_source(&mut message)?.check_component(),
        Err(Error::SourceComponent)
    ));

    Ok(())
}

#[tokio::test]
async fn check_valid() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("component valid *(should be cloned with the component)*")?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![valid_component()],
        })])?
        .await?
        .model()
        .await?;

    ctx.message_source(&mut message)?
        .check_component()?
        .create()
        .await?;

    Ok(())
}

fn valid_component() -> Component {
    Component::Button(Button {
        label: Some("wöæo".to_owned()),
        url: Some("https://youtube.com/shorts/e7nRorYvy-Q".to_owned()),
        style: ButtonStyle::Link,
        disabled: false,
        emoji: None,
        custom_id: None,
    })
}

fn invalid_component() -> Component {
    Component::Button(Button {
        custom_id: Some("dont click".to_owned()),
        label: Some("why do i exist if im disabled".to_owned()),
        style: ButtonStyle::Primary,
        disabled: true,
        emoji: None,
        url: None,
    })
}
