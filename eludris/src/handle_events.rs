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

pub async fn handle_events(conn: Connection, gateway: GatewayClient, url: String) -> Result<()> {
    let conn = Arc::new(Mutex::new(conn));
    let mut events = gateway.get_events().await?;

    while let Some(msg) = events.next().await {
        let conn = Arc::clone(&conn);
        let url = url.clone();
        tokio::spawn(async move {
            if !msg.author.starts_with("Bridge-") {
                let event = Event {
                    platform: "eludris",
                    identifier: url.clone(),
                    data: EventData::MessageCreate(Message {
                        content: msg.content,
                        author: msg.author,
                        attachments: Vec::new(),
                        replies: Vec::new(),
                        // :(
                        avatar: Some(ELUDRIS_AVATAR.to_string()),
                    }),
                };

                let mut conn: tokio::sync::MutexGuard<Connection> = conn.lock().await;
                let channel_name = conn
                    .get::<String, Option<String>>(format!("eludris:key:{}", url))
                    .await
                    .unwrap();
                match channel_name {
                    Some(channel_name) => conn
                        .publish::<String, String, ()>(
                            channel_name,
                            serde_json::to_string(&event).unwrap(),
                        )
                        .await
                        .unwrap(),
                    None => {
                        log::debug!("Ignoring channel {}", url);
                    }
                }
            }
        });
    }

    Ok(())
}
