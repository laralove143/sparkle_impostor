use twilight_http::{api_error::ApiError, Client};
use twilight_model::{
    channel::{ChannelType, Message},
    id::{marker::ChannelMarker, Id},
};

use crate::{error::Error, tests::Context, MessageSource};

pub(crate) async fn create_source_thread(
    http: &Client,
    channel_id: Id<ChannelMarker>,
) -> Result<Id<ChannelMarker>, anyhow::Error> {
    let thread = http
        .create_thread(
            channel_id,
            "sparkle impostor create later messages ratelimit source",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?;

    http.create_message(thread.id)
        .content(
            "create later messages ratelimit *(this and messages below should be cloned to \
             another thread in order)*",
        )?
        .await?
        .model()
        .await?;

    for n in 1..=200_u8 {
        for i in 0..=3_u8 {
            match http
                .create_message(thread.id)
                .content(&n.to_string())?
                .await
            {
                Ok(_) => break,
                Err(err)
                    if matches!(
                        err.kind(),
                        twilight_http::error::ErrorType::Response {
                            error: ApiError::Ratelimited(_),
                            ..
                        }
                    ) =>
                {
                    if i == 3 {
                        return Err(err.into());
                    }
                    continue;
                }
                Err(err) => return Err(err.into()),
            }
        }
    }

    Ok(thread.id)
}

#[tokio::test]
async fn error() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("check message not last *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    ctx.create_message()
        .content("message after *(should not be cloned)*")?
        .await?;

    assert!(matches!(
        MessageSource::from_message(&message)?
            .check_not_last(&ctx.http)
            .await,
        Err(Error::SourceNotLast)
    ));

    Ok(())
}

#[tokio::test]
async fn create() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("create later messages *(this and messages below should be cloned in order)*")?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;
    ctx.create_message().content("3")?.await?;

    MessageSource::from_message(&message)?
        .create(&ctx.http)
        .await?
        .create_later_messages(&ctx.http, None)
        .await?;

    Ok(())
}

#[tokio::test]
async fn limit_under() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content(
            "create later messages limit under *(this should be cloned but not the messages \
             below)*",
        )?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;
    ctx.create_message().content("3")?.await?;

    assert!(matches!(
        MessageSource::from_message(&message)?
            .create(&ctx.http)
            .await?
            .create_later_messages(&ctx.http, Some(2))
            .await,
        Err(Error::SourceBeforeLimit)
    ));

    Ok(())
}

#[tokio::test]
async fn limit_over() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content(
            "create later messages limit over *(this and messages below should be cloned in \
             order)*",
        )?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;
    ctx.create_message().content("3")?.await?;

    MessageSource::from_message(&message)?
        .create(&ctx.http)
        .await?
        .create_later_messages(&ctx.http, Some(3))
        .await?;

    Ok(())
}

#[tokio::test]
async fn ratelimit() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut messages: Vec<Message> = vec![];
    loop {
        let get_messages = ctx
            .http
            .channel_messages(ctx.not_last_source_thread_id)
            .limit(100)?;

        let mut message_batch = if messages.is_empty() {
            get_messages.await?.models().await?
        } else {
            get_messages
                .before(messages.last().unwrap().id)
                .await?
                .models()
                .await?
        };

        let is_done = message_batch.is_empty() || message_batch.len() % 100 != 0;

        messages.append(&mut message_batch);

        if is_done {
            break;
        }
    }

    let message = messages.last_mut().unwrap();

    let mut message_source = MessageSource::from_message(message)?;
    message_source.channel_id = ctx
        .http
        .create_thread(
            ctx.channel_id,
            "sparkle impostor create later messages target",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?
        .id;

    message_source
        .handle_thread(&ctx.http)
        .await?
        .create(&ctx.http)
        .await?
        .create_later_messages(&ctx.http, None)
        .await?;

    Ok(())
}
