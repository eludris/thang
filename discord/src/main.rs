mod handle_events;
mod handle_redis;

use models::ThangResult;
use std::env;
use twilight_gateway::{CloseFrame, Intents, Shard, ShardId};
use twilight_http::Client;
use twilight_model::{
    channel::message::AllowedMentions,
    id::{marker::ChannelMarker, Id},
};

const WEBHOOK_NAME: &str = "Eludris Bridge";
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("DISCORD_TOKEN")
        .expect("Could not find the \"DISCORD_TOKEN\" environment variable");
    let bridge_channel_id = serde_json::from_str::<Id<ChannelMarker>>(
        &env::var("DISCORD_CHANNEL_ID")
            .expect("Could not find the \"DISCORD_CHANNEL_ID\" environment variable"),
    )
    .expect(
        "Could not deserialize the \"DISCORD_CHANNEL_ID\" environment variable as a valid Discord ID",
    );
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let webhook_name =
        env::var("DISCORD_WEBHOOK_NAME").unwrap_or_else(|_| WEBHOOK_NAME.to_string());

    let redis = redis::Client::open(redis_url)?;

    // Get Discord ready.
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    log::info!("Connected to Discord");

    let http = Client::builder()
        .token(token)
        .default_allowed_mentions(AllowedMentions::default())
        .build();

    // Find webhook, if does not exist then create.
    let webhooks = http
        .channel_webhooks(bridge_channel_id)
        .await?
        .models()
        .await?;

    let webhook = webhooks
        .into_iter()
        .find(|w| w.name.as_ref() == Some(&webhook_name));

    let webhook = match webhook {
        Some(webhook) => webhook,
        None => {
            http.create_webhook(bridge_channel_id, &webhook_name)
                .unwrap()
                .await?
                .model()
                .await?
        }
    };

    log::info!("Found webhook {:?}", webhook.id.to_string());

    let err = tokio::select!(
        e  = handle_events::handle_events(&mut shard, redis.get_async_connection().await?, webhook.id) => {
            log::error!("Events failed first {:?}", e);
        e
        },
        e = handle_redis::handle_redis(redis.get_async_connection().await?, http, webhook) => {
            log::error!("Receive failed first {:?}", e);
        e
        },
    );

    shard
        .close(CloseFrame {
            code: 1001,
            reason: std::borrow::Cow::Borrowed("Shutting down"),
        })
        .await?;

    err
}
