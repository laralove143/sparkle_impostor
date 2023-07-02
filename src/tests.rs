use std::env;

use dotenvy::dotenv;
use twilight_http::{request::channel::message::CreateMessage, Client};
use twilight_model::{
    channel::Message,
    guild::Member,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

use crate::{error::Error, MessageSource};

mod avatar;
mod forum;
mod not_last;
mod thread;

struct Context {
    http: Client,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    forum_channel_id: Id<ChannelMarker>,
    not_last_source_thread_id: Id<ChannelMarker>,
    member: Member,
    owner: Member,
}

impl Context {
    async fn new() -> Self {
        dotenv().unwrap();

        let http = Client::new(env::var("BOT_TOKEN").unwrap());
        let channel_id = env::var("CHANNEL_ID").unwrap().parse().unwrap();
        let forum_channel_id = env::var("FORUM_CHANNEL_ID").unwrap().parse().unwrap();
        let not_last_source_thread_id = if let Ok(var) = env::var("NOT_LAST_SOURCE_THREAD_ID") {
            var.parse().unwrap()
        } else {
            not_last::create_source_thread(&http, channel_id)
                .await
                .unwrap()
        };

        let guild_id = http
            .channel(channel_id)
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
            .guild_id
            .unwrap();

        let member = http
            .guild_member(
                guild_id,
                http.current_user().await.unwrap().model().await.unwrap().id,
            )
            .await
            .unwrap()
            .model()
            .await
            .unwrap();

        let owner = http
            .guild_member(
                guild_id,
                http.current_user_application()
                    .await
                    .unwrap()
                    .model()
                    .await
                    .unwrap()
                    .owner
                    .unwrap()
                    .id,
            )
            .await
            .unwrap()
            .model()
            .await
            .unwrap();

        Self {
            http,
            guild_id,
            channel_id,
            forum_channel_id,
            not_last_source_thread_id,
            member,
            owner,
        }
    }

    const fn create_message(&self) -> CreateMessage<'_> {
        self.http.create_message(self.channel_id)
    }

    async fn clone_message(&self, message: &Message) -> Result<(), Error> {
        MessageSource::from_message(message, &self.http)?
            .create()
            .await?;

        Ok(())
    }
}

#[tokio::test]
async fn basic() -> Result<(), anyhow::Error> {
    let ctx = Context::new().await;

    let message = ctx
        .create_message()
        .content("basic")?
        .await?
        .model()
        .await?;

    ctx.clone_message(&message).await?;

    Ok(())
}
