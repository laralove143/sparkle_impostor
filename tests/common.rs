use std::env;

use dotenvy::dotenv;
use sparkle_impostor::{error::Error, MessageSource};
use twilight_http::{request::channel::message::CreateMessage, Client};
use twilight_model::{
    channel::Message,
    guild::Member,
    id::{
        marker::{ChannelMarker, EmojiMarker, GuildMarker},
        Id,
    },
};

pub struct Context {
    pub http: Client,
    pub guild_id: Id<GuildMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub forum_channel_id: Id<ChannelMarker>,
    pub not_last_source_thread_id: Id<ChannelMarker>,
    pub guild_emoji_id: Id<EmojiMarker>,
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
        let guild_emoji_id = env::var("GUILD_EMOJI_ID")?.parse()?;

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
            guild_emoji_id,
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
        message: &'a mut Message,
    ) -> Result<MessageSource<'a>, anyhow::Error> {
        message.guild_id = Some(self.guild_id);

        Ok(MessageSource::from_message(message, &self.http)?)
    }

    #[allow(dead_code)]
    pub async fn clone_message(&self, message: &mut Message) -> Result<(), Error> {
        message.guild_id = Some(self.guild_id);

        MessageSource::from_message(message, &self.http)?
            .create()
            .await?;

        Ok(())
    }
}

#[allow(dead_code)]
pub async fn create_later_messages(
    mut message_source: MessageSource<'_>,
) -> Result<(), anyhow::Error> {
    let later_messages = message_source.later_messages().await?;
    assert!(later_messages.iter().all(Result::is_ok));

    for later_message in later_messages {
        later_message?.create().await?;
    }

    Ok(())
}

async fn create_not_last_source_thread(
    http: &Client,
    channel_id: Id<ChannelMarker>,
) -> Result<Id<ChannelMarker>, anyhow::Error> {
    let message = http
        .create_message(channel_id)
        .content(
            "create later messages *(this and messages below should be cloned to another thread \
             in order)*",
        )?
        .await?
        .model()
        .await?;

    http.create_thread_from_message(
        channel_id,
        message.id,
        "sparkle impostor create later messages source",
    )?
    .await?;

    let thread_id = message.id.cast();
    for n in 1..=50_u8 {
        for i in 0..=3_u8 {
            match http
                .create_message(thread_id)
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

    Ok(thread_id)
}

fn _source_construct() {
    drop(MessageSource {
        source_id: Id::new(1),
        source_channel_id: Id::new(1),
        source_thread_id: None,
        content: String::new(),
        embeds: &[],
        tts: false,
        flags: None,
        channel_id: Id::new(1),
        guild_id: Id::new(1),
        guild_emoji_ids: None,
        username: String::new(),
        avatar_info: sparkle_impostor::avatar::Info {
            url: None,
            user_id: Id::new(1),
            guild_id: Id::new(1),
            user_discriminator: 1,
            user_avatar: None,
            member_avatar: None,
        },
        webhook_name: String::new(),
        sticker_info: sparkle_impostor::sticker::Info { exists: false },
        reaction_info: sparkle_impostor::reaction::Info { reactions: &[] },
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
        later_messages: sparkle_impostor::later_messages::Info {
            messages: vec![],
            is_complete: false,
            is_source_created: false,
            is_later_message_sources_created: false,
        },
        webhook: None,
        response: None,
        http: &Client::new(String::new()),
    });
}
