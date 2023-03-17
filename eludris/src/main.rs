mod handle_redis;
mod handle_websocket;

use eludrs::{GatewayClient, HttpClient};
use models::Result;
use std::env;

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";
const DEFAULT_REST_URL: &str = "https://api.eludris.gay";
const DEFAULT_GATEWAY_URL: &str = "wss://ws.eludris.gay/";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());

    let redis = redis::Client::open(redis_url)?;
    log::info!("Connected to Redis {}", redis.get_connection_info().addr);

    let rest_url = env::var("ELUDRIS_REST_URL").unwrap_or_else(|_| DEFAULT_REST_URL.to_string());
    let gateway_url =
        env::var("ELUDRIS_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string());

    let rest = HttpClient::new().rest_url(rest_url);
    let gateway = GatewayClient::new().gateway_url(gateway_url);

    let err = tokio::select! {
        e = handle_redis::handle_redis(redis.get_async_connection().await?, rest) => {
            log::error!("Events failed first {:?}", e);
            e
        },
        e = handle_websocket::handle_websocket(redis.get_async_connection().await?, gateway) => {
            log::error!("Websocket failed first {:?}", e);
            e
        },
    };

    err
}
