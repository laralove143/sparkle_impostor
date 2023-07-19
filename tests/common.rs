use std::env;

use dotenvy::dotenv;
use sparkle_impostor::{error::Error, MessageSource};
use twilight_http::{request::channel::message::CreateMessage, Client};
use twilight_model::{
    channel::{ChannelType, Message},
    guild::Member,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

pub struct Context {
    pub http: Client,
    pub guild_id: Id<GuildMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub forum_channel_id: Id<ChannelMarker>,
    pub not_last_source_thread_id: Id<ChannelMarker>,
    pub member: Member,
    pub owner: Member,
}

impl Context {
    pub async fn new() -> Self {
        Self::new_fallible().await.unwrap()
    }

    async fn new_fallible() -> Result<Context, anyhow::Error> {
        dotenv()?;

        let http = Client::new(env::var("BOT_TOKEN")?);
        let channel_id = env::var("CHANNEL_ID")?.parse()?;
        let forum_channel_id = env::var("FORUM_CHANNEL_ID")?.parse()?;
        let not_last_source_thread_id = if let Ok(var) = env::var("NOT_LAST_SOURCE_THREAD_ID") {
            var.parse()?
        } else {
            create_not_last_source_thread(&http, channel_id).await?
        };

        let guild_id = http
            .channel(channel_id)
            .await?
            .model()
            .await?
            .guild_id
            .unwrap();

        let member = http
            .guild_member(guild_id, http.current_user().await?.model().await?.id)
            .await?
            .model()
            .await?;

        let owner = http
            .guild_member(
                guild_id,
                http.current_user_application()
                    .await?
                    .model()
                    .await?
                    .owner
                    .unwrap()
                    .id,
            )
            .await?
            .model()
            .await?;

        Ok(Self {
            http,
            guild_id,
            channel_id,
            forum_channel_id,
            not_last_source_thread_id,
            member,
            owner,
        })
    }

    #[allow(dead_code)]
    pub const fn create_message(&self) -> CreateMessage<'_> {
        self.http.create_message(self.channel_id)
    }

    #[allow(dead_code)]
    pub fn message_source<'a>(
        &'a self,
        message: &'a Message,
    ) -> Result<MessageSource<'a>, anyhow::Error> {
        Ok(MessageSource::from_message(message, &self.http)?)
    }

    #[allow(dead_code)]
    pub async fn clone_message(&self, message: &Message) -> Result<(), Error> {
        MessageSource::from_message(message, &self.http)?
            .create()
            .await?;

        Ok(())
    }
}

async fn create_not_last_source_thread(
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

    for n in 1..=50_u8 {
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
                            error: twilight_http::api_error::ApiError::Ratelimited(_),
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

fn _source_construct() {
    drop(MessageSource {
        source_id: Id::new(1),
        source_channel_id: Id::new(1),
        content: String::new(),
        embeds: &[],
        tts: false,
        flags: None,
        channel_id: Id::new(1),
        username: String::new(),
        avatar_url: String::new(),
        webhook_name: String::new(),
        sticker_info: sparkle_impostor::sticker::Info { exists: false },
        attachment_info: sparkle_impostor::attachment::Info {
            attachments: &[],
            #[cfg(feature = "upload")]
            attachments_upload: vec![],
        },
        component_info: sparkle_impostor::component::Info {
            url_components: vec![],
            has_invalid_components: false,
        },
        thread_info: sparkle_impostor::thread::Info::Unknown,
        later_messages: sparkle_impostor::not_last::Info {
            messages: vec![],
            is_complete: false,
            is_source_created: false,
            is_later_message_sources_created: false,
        },
        webhook: None,
        http: &Client::new(String::new()),
    });
}
