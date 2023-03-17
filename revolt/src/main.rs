mod handle_events;
mod handle_redis;

use std::env;

use models::Result;
use revolt_wrapper::{GatewayClient, HttpClient};

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let channel_id = env::var("REVOLT_CHANNEL_ID").expect("REVOLT_CHANNEL_ID must be set");

    let token = env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN must be set");

    let client = GatewayClient::new(token.clone());
    let http = HttpClient::new(token);

    let mut events = client.get_events().await?;

    log::info!("Connected to gateway!");

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());

    let redis = redis::Client::open(redis_url)?;
    log::info!("Connected to Redis {}", redis.get_connection_info().addr);

    let bot_id = http.fetch_self().await?.id;

    let err = tokio::select! {
        e = handle_redis::handle_redis(redis.get_async_connection().await?, http, channel_id.clone()) => {
            log::error!("Events failed first {:?}", e);
            e
        },
        e = handle_events::handle_events(&mut events, redis.get_async_connection().await?, channel_id, bot_id) => {
            log::error!("Websocket failed first {:?}", e);
            e
        },
    };

    err
}
