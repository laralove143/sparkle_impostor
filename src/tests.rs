use std::env;

use dotenvy::dotenv;
use twilight_http::{request::channel::message::CreateMessage, Client};
use twilight_model::{
    guild::Member,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

use crate::clone_message;

mod avatar;
mod thread;

struct Context {
    http: Client,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    owner: Member,
}

impl Context {
    async fn new() -> Self {
        dotenv().unwrap();

        let http = Client::new(env::var("BOT_TOKEN").unwrap());
        let channel_id = env::var("CHANNEL_ID").unwrap().parse().unwrap();
        let guild_id = http
            .channel(channel_id)
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
            .guild_id
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
            owner,
        }
    }

    const fn create_message(&self) -> CreateMessage<'_> {
        self.http.create_message(self.channel_id)
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

    clone_message(&message, &ctx.http).await?;

    Ok(())
}
