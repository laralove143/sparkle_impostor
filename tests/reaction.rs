use sparkle_impostor::error::Error;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
    channel::message::{Reaction, ReactionType},
    id::Id,
};

use crate::common::Context;

mod common;

const REACTION_EXTERNAL_EMOJI: Reaction = Reaction {
    count: 0,
    emoji: ReactionType::Custom {
        id: Id::new(1107789267696095232),
        animated: false,
        name: None,
    },
    me: false,
};

#[tokio::test]
async fn handle() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message_id = ctx
        .create_message()
        .content("reaction handle *(should be cloned with reactions)*")?
        .await?
        .model()
        .await?
        .id;

    ctx.http
        .create_reaction(
            ctx.channel_id,
            message_id,
            &RequestReactionType::Unicode { name: "😭" },
        )
        .await?;

    ctx.http
        .create_reaction(
            ctx.channel_id,
            message_id,
            &RequestReactionType::Custom {
                id: ctx.guild_emoji_id,
                name: None,
            },
        )
        .await?;

    let mut message = ctx
        .http
        .message(ctx.channel_id, message_id)
        .await?
        .model()
        .await?;

    message.reactions.push(REACTION_EXTERNAL_EMOJI);

    ctx.message_source(&mut message)?
        .create()
        .await?
        .handle_reaction()
        .await?;

    Ok(())
}

#[tokio::test]
async fn check_not_external() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("reaction check not external *(should not be cloned)*")?
        .await?
        .model()
        .await?;

    message.reactions.push(REACTION_EXTERNAL_EMOJI);

    assert!(matches!(
        ctx.message_source(&mut message)?
            .check_reaction(sparkle_impostor::reaction::CheckBehavior::NotExternal)
            .await,
        Err(Error::SourceReactionExternal)
    ));

    Ok(())
}

#[tokio::test]
async fn before_create() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message_id = ctx
        .create_message()
        .content("reaction handle before create *(should not be cloned)*")?
        .await?
        .model()
        .await?
        .id;

    ctx.http
        .create_reaction(
            ctx.channel_id,
            message_id,
            &RequestReactionType::Unicode { name: "😭" },
        )
        .await?;

    let mut message = ctx
        .http
        .message(ctx.channel_id, message_id)
        .await?
        .model()
        .await?;

    assert!(matches!(
        ctx.message_source(&mut message)?.handle_reaction().await,
        Err(Error::NotCreated)
    ));

    Ok(())
}
