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
            "sparkle impostor create later messages source",
            ChannelType::PublicThread,
        )?
        .await?
        .model()
        .await?;

    http.create_message(thread.id)
        .content(
            "create later messages *(this and messages below should be cloned to another thread \
             in order)*",
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
async fn check_err() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("check is in last err *(nothing should be cloned)*")?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;

    let res = MessageSource::from_message(&message, &ctx.http)?
        .check_is_in_last(2)
        .await;
    assert!(matches!(res, Err(Error::SourceNotIn(2))));

    Ok(())
}

#[tokio::test]
async fn check_ok() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("check is in ok *(nothing should be cloned)*")?
        .await?
        .model()
        .await?;

    ctx.create_message().content("1")?.await?;
    ctx.create_message().content("2")?.await?;

    MessageSource::from_message(&message, &ctx.http)?
        .check_is_in_last(3)
        .await?;

    Ok(())
}

#[tokio::test]
async fn create_later() -> Result<(), anyhow::Error> {
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

    let mut message_source = MessageSource::from_message(message, &ctx.http)?;
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
        .handle_thread()
        .await?
        .create()
        .await?
        .create_later_messages()
        .await?;

    Ok(())
}
