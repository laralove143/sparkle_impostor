use twilight_model::guild::{MemberFlags, PartialMember};

use crate::tests::Context;

#[tokio::test]
async fn default() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let mut message = ctx
        .create_message()
        .content("no avatar")?
        .await?
        .model()
        .await?;

    message.author.avatar = None;
    if let Some(member) = &mut message.member {
        member.avatar = None;
    }

    ctx.clone_message(&message).await;

    Ok(())
}

#[tokio::test]
async fn non_animated() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let avatar = match ctx.owner.user.avatar {
        Some(avatar) if !avatar.is_animated() => avatar,
        _ => {
            ctx.create_message()
                .content(
                    "can't test non-animated image avatars, bot's owner doesn't have a \
                     non-animated image avatar",
                )?
                .await?;
            return Ok(());
        }
    };

    let mut message = ctx
        .create_message()
        .content("non-animated avatar *(should be bot owner's avatar)*")?
        .await?
        .model()
        .await?;

    message.author.id = ctx.owner.user.id;
    message.author.avatar = Some(avatar);

    ctx.clone_message(&message).await;

    Ok(())
}

#[tokio::test]
async fn animated() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let avatar = match ctx.owner.user.avatar {
        Some(avatar) if avatar.is_animated() => avatar,
        _ => {
            ctx.create_message()
                .content(
                    "can't test animated avatars, bot's owner doesn't have an animated avatar",
                )?
                .await?;
            return Ok(());
        }
    };

    let mut message = ctx
        .create_message()
        .content("animated avatar *(should be bot owner's avatar but not animated)*")?
        .await?
        .model()
        .await?;

    message.author.id = ctx.owner.user.id;
    message.author.avatar = Some(avatar);

    ctx.clone_message(&message).await;

    Ok(())
}

#[tokio::test]
async fn guild() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let Some(avatar) = ctx.owner.avatar else {
        ctx.create_message()
            .content(
                "can't test guild avatars, bot's owner doesn't have a guild avatar in this guild"
            )?
            .await?;
        return Ok(());
    };

    let mut message = ctx
        .create_message()
        .content("guild avatar *(should be bot owner's guild avatar)*")?
        .await?
        .model()
        .await?;

    message.author.id = ctx.owner.user.id;
    message.guild_id = Some(ctx.guild_id);
    message.member = Some(PartialMember {
        avatar: Some(avatar),
        communication_disabled_until: None,
        deaf: false,
        flags: MemberFlags::empty(),
        joined_at: ctx.owner.joined_at,
        mute: false,
        nick: None,
        permissions: None,
        premium_since: None,
        roles: vec![],
        user: None,
    });

    ctx.clone_message(&message).await;

    Ok(())
}
