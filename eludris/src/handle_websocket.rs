use std::sync::Arc;

use eludrs::todel::Payload;
use eludrs::GatewayClient;
use futures::StreamExt;
use models::Event;
use models::ThangResult;
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;

pub async fn handle_websocket(redis: Connection, gateway: GatewayClient) -> ThangResult<()> {
    let redis = Arc::new(Mutex::new(redis));
    let mut events = gateway.get_events().await?;

    while let Some(msg) = events.next().await {
        let redis = Arc::clone(&redis);
        tokio::spawn(async move {
            if !msg.author.starts_with("Bridge-") {
                let event = Event::Eludris(Payload::MessageCreate(msg));
                redis
                    .lock()
                    .await
                    .publish::<&str, String, ()>(
                        "thang-bridge",
                        serde_json::to_string(&event).unwrap(),
                    )
                    .await
                    .unwrap();
            }
        });
    }

    Ok(())
}
