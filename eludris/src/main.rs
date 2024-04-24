mod handle_events;
mod handle_redis;

use eludrs::{GatewayClient, HttpClient};
use futures::future::{abortable, select_all};
use futures_util::FutureExt;
use models::{Config, Result};
use redis::AsyncCommands;
use std::{collections::HashMap, env};

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string("./config.yml")?)?;

    let redis = redis::Client::open(redis_url)?;
    log::info!("Connected to Redis {}", redis.get_connection_info().addr);

    let mut conn = redis.get_multiplexed_async_connection().await?;
    for channel in &config {
        if let Some(eludris) = &channel.eludris {
            for url in eludris {
                conn.set(format!("eludris:key:{}", url), &channel.name)
                    .await?;
            }
            conn.sadd(format!("eludris:instances:{}", channel.name), eludris)
                .await?;
        }
    }

    let mut clients: HashMap<String, HttpClient> = HashMap::new();
    let mut gateway: HashMap<String, GatewayClient> = HashMap::new();

    for url in config
        .iter()
        .filter_map(|config| config.eludris.as_ref())
        .flatten()
    {
        if clients.contains_key(url) {
            continue;
        }
        let url = url
            .replace("localhost", "172.17.0.1")
            .replace("127.0.0.1", "172.17.0.1");
        let client = HttpClient::new().rest_url(url.to_string());
        let gateway_url = client
            .fetch_instance_info()
            .await?
            .pandemonium_url
            .replace("localhost", "172.17.0.1");

        clients.insert(url.to_string(), client);
        gateway.insert(
            url.to_string(),
            GatewayClient::new().gateway_url(gateway_url),
        );
    }

    let mut futures = Vec::new();

    futures.push(
        handle_redis::handle_redis(
            redis.get_async_pubsub().await?,
            redis.get_multiplexed_async_connection().await?,
            clients,
            config,
        )
        .boxed(),
    );

    for (url, gw) in gateway {
        futures.push(
            handle_events::handle_events(redis.get_multiplexed_async_connection().await?, gw, url)
                .boxed(),
        );
    }

    let (output, index, futures) = select_all(futures).await;

    if index == 0 {
        log::error!("Redis failed first: {:?}", output);
    } else {
        log::error!("Events failed first: {:?}", output);
    }

    for future in futures {
        let (res, abort_handle) = abortable(future.map(Result::Ok));
        tokio::spawn(async move {
            let _ = res.await;
        });
        abort_handle.abort();
    }

    output
}
