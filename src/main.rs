mod eludris;
mod events;
mod types;

use dotenv::dotenv;
use futures::StreamExt;
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Intents,
};
use twilight_http::Client;
use twilight_model::{channel::message::AllowedMentions, id::Id};
use types::{Context, ThangResult};

const DEFAULT_REST_URL: &str = "https://eludris.tooty.xyz";
const DEFAULT_GATEWAY_URL: &str = "wss://eludris.tooty.xyz/ws/";
const WEBHOOK_NAME: &str = "Eludris Bridge";

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv().ok();
    env_logger::init();

    let token = env::var("TOKEN")?;

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

    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let eludris_gateway_url =
        env::var("ELUDRIS_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string());
    let (eludris_ws_writer, eludris_ws_reader) =
        connect_async(&eludris_gateway_url).await?.0.split();

    let eludris_ws_writer = Mutex::new(eludris_ws_writer);

    let http = Client::builder()
        .token(token)
        .default_allowed_mentions(AllowedMentions::default())
        .build();
    let bridge_channel_id = Id::new(env::var("BRIDGE_CHANNEL_ID")?.parse::<u64>()?);

    let webhooks = http
        .channel_webhooks(bridge_channel_id)
        .exec()
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
                .exec()
                .await?
                .model()
                .await?
        }
    };

    let context = Arc::new(Context {
        http,
        bridge_webhook_id: webhook.id,
        bridge_webhook_token: webhook.token.unwrap(),
        bridge_channel_id,
        eludris_rest_url: env::var("ELUDRIS_REST_URL")
            .unwrap_or_else(|_| DEFAULT_REST_URL.to_string()),
        eludris_http_client: reqwest::Client::new(),
        eludris_gateway_url,
        eludris_ws_writer,
    });

    let err = tokio::select! {
        e = events::iterate_websocket(event_iterator, context.clone()) => {
            log::error!("Discord failed first {:?}", e);
            e
        }
        e = eludris::iterate_websocket(eludris_ws_reader, context.clone()) => {
            log::error!("Eludris failed first {:?}", e);
            e
        }
    };

    err
}
