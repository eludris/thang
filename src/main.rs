mod events;
mod types;

use dotenv::dotenv;
use events::handle_event;
use futures::stream::StreamExt;
use std::{env, sync::Arc};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Intents,
};
use twilight_http::Client;
use types::{Context, ThangResult};
use url::Url;

const DEFAULT_REST_URL: &str = "https://eludris.tooty.xyz";
const DEFAULT_GATEWAY_URL: &str = "wss://eludris.tooty.xyz/ws/";

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv().ok();
    env_logger::init();

    let token = env::var("TOKEN")?;
    let webhook_url = env::var("WEBHOOK_URL")?;

    let scheme = ShardScheme::Range {
        from: 0,
        to: 0,
        total: 1,
    };

    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let (cluster, mut events) = Cluster::builder(token.clone(), intents)
        .shard_scheme(scheme)
        .build()
        .await?;

    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let context = Arc::new(Context {
        http: Client::new(token),
        webhook_url: webhook_url.clone(),
        webhook_id: Url::parse(&webhook_url)?
            .path_segments()
            .unwrap()
            .next()
            .unwrap()
            .parse::<u64>()?,
        bridge_channel_id: env::var("BRIDGE_CHANNEL_ID")?.parse::<u64>()?,
        eludris_gateway_url: env::var("ELUDRIS_GATEWAY_URL")
            .unwrap_or(DEFAULT_GATEWAY_URL.to_string()),
        eludris_rest_url: env::var("ELUDRIS_REST_URL").unwrap_or(DEFAULT_REST_URL.to_string()),
    });

    while let Some((shard_id, event)) = events.next().await {
        tokio::spawn(handle_event(shard_id, event, context.clone()));
    }

    Ok(())
}
