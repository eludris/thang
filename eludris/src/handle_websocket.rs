use std::sync::Arc;

use eludrs::GatewayClient;
use futures::StreamExt;
use models::Event;
use models::EventData;
use models::Message;
use models::Result;
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;

const ELUDRIS_AVATAR: &str =
    "https://raw.githubusercontent.com/eludris/.github/main/assets/das_ding.png";

pub async fn handle_websocket(redis: Connection, gateway: GatewayClient) -> Result<()> {
    let redis = Arc::new(Mutex::new(redis));
    let mut events = gateway.get_events().await?;

    while let Some(msg) = events.next().await {
        let redis = Arc::clone(&redis);
        tokio::spawn(async move {
            if !msg.author.starts_with("Bridge-") {
                let event = Event {
                    platform: "eludris",
                    data: EventData::MessageCreate(Message {
                        content: msg.content,
                        author: msg.author,
                        attachments: Vec::new(),
                        replies: Vec::new(),
                        // :(
                        avatar: Some(ELUDRIS_AVATAR.to_string()),
                    }),
                };
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
