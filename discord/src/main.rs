mod events;
mod receive;
mod types;
mod util;

use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Intents,
};
use twilight_http::Client;
use twilight_model::{channel::message::AllowedMentions, id::Id};
use types::{Context, ThangResult};

const WEBHOOK_NAME: &str = "Eludris Bridge";
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Start up ~~redis~~ keydb
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let redis = redis::Client::open(redis_url)?;
    let redis_connection = redis.get_connection()?;
    log::info!("Connected to redis {:?}", redis.get_connection_info().addr);

    // Get Discord ready.
    let token = env::var("DISCORD_TOKEN")?;

    let scheme = ShardScheme::Range {
        from: 0,
        to: 0,
        total: 1,
    };

    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let (cluster, event_iterator) = Cluster::builder(token.clone(), intents)
        .shard_scheme(scheme)
        .build()
        .await?;

    log::info!("Connected to Discord");

    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = Client::builder()
        .token(token)
        .default_allowed_mentions(AllowedMentions::default())
        .build();
    let bridge_channel_id = Id::new(env::var("DISCORD_CHANNEL_ID")?.parse::<u64>()?);

    // Find webhook, if does not exist then create.
    let webhooks = http
        .channel_webhooks(bridge_channel_id)
        .await?
        .models()
        .await?;

    let blank = String::from("");
    let webhook = webhooks
        .into_iter()
        .find(|w| w.name.as_ref().unwrap_or(&blank) == &WEBHOOK_NAME.to_string());

    let webhook = match webhook {
        Some(webhook) => webhook,
        None => {
            http.create_webhook(bridge_channel_id, WEBHOOK_NAME)
                .unwrap()
                .await?
                .model()
                .await?
        }
    };

    log::info!("Found webhook {:?}", webhook.id.to_string());

    let context = Arc::new(RwLock::new(Context {
        http,
        bridge_webhook_id: webhook.id,
        bridge_webhook_token: webhook.token.unwrap(),
        bridge_channel_id,
        redis: redis_connection,
    }));

    let err = tokio::select!(
        e  = events::iterate_websocket(event_iterator, context.clone()) => {
            log::error!("Events failed first {:?}", e);
        e
        },
        e = receive::receive_redis(redis, context.clone()) => {
            log::error!("Receive failed first {:?}", e);
        e
        },
    );

    err
}
