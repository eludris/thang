mod handle_redis;

use models::ThangResult;
use reqwest::Client;
use std::env;
use twilight_model::id::{marker::ChannelMarker, Id};

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";
const DEFAULT_REST_URL: &str = "https://eludris.tooty.xyz";
// const DEFAULT_GATEWAY_URL: &str = "wss://eludris.tooty.xyz/ws/";

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // FIXME: Temporary single channel bridge until eludris channels.
    let discord_bridge_channel_id = serde_json::from_str::<Id<ChannelMarker>>(
        &env::var("DISCORD_CHANNEL_ID")
            .expect("Could not find the \"DISCORD_CHANNEL_ID\" environment variable"),
    )
    .expect(
        "Could not deserialize the \"DISCORD_CHANNEL_ID\" environment variable as a valid Discord ID",
    );

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());

    let redis = redis::Client::open(redis_url)?;
    log::info!("Connected to Redis {}", redis.get_connection_info().addr);

    let rest_url = env::var("ELUDRIS_REST_URL").unwrap_or_else(|_| DEFAULT_REST_URL.to_string());
    // let gateway_url =
    //     env::var("ELUDRIS_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string());
    // let (writer, reader) = connect_async(&gateway_url).await?.0.split();

    // let writer = Mutex::new(writer);

    let client = Client::new();

    let err = tokio::select! {
        e = handle_redis::handle_redis( redis.get_async_connection().await?, client, rest_url, discord_bridge_channel_id)=> {
            log::error!("Events failed first {:?}", e);
        e
        },
        // e = handle_websocket::handle_websocket(redis.get_async_connection().await?, reader) => {
        //     log::error!("Websocket failed first {:?}", e);
        //     e},
        // e = ping_websocket::ping_websocket(writer) => {
        //     log::error!("Websocket failed first {:?}", e);
        //     e},
    };

    err
}
