mod events;
mod types;

use crate::events::handle_event;
use dotenv::dotenv;
use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Intents,
};
use twilight_http::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().ok();
    env_logger::init();

    let token = env::var("TOKEN")?;

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

    // The http client is seperate from the gateway, so startup a new
    // one, also use Arc such that it can be cloned to other threads.
    let http = Arc::new(Client::new(token));

    while let Some((shard_id, event)) = events.next().await {
        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }

    Ok(())
}
