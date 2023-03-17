use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use models::{Event, Message};
use models::{EventData, Result};
use redis::{aio::Connection, AsyncCommands};
use revolt_wrapper::gateway::Events;
use revolt_wrapper::Event as GatewayEvent;
use tokio::sync::Mutex;

pub async fn handle_events(
    events: &mut Events,
    conn: Connection,
    channel_id: String,
    bot_id: String,
) -> Result<()> {
    let conn = Arc::new(Mutex::new(conn));
    while let Some(event) = events.next().await {
        let conn = Arc::clone(&conn);
        let bot_id = bot_id.clone();
        let channel_id = channel_id.clone();
        tokio::spawn(async move {
            let payload = match event {
                GatewayEvent::Message(msg) => {
                    if (msg.author == bot_id && msg.masquerade.is_some())
                        || msg.channel != channel_id
                    {
                        return;
                    }

                    EventData::MessageCreate(Message {
                        content: msg.content.unwrap_or("".to_string()),
                        author: msg.author,
                        attachments: match msg.attachments {
                            Some(attachments) => attachments
                                .into_iter()
                                .map(|a| a.url())
                                .collect::<Vec<String>>(),
                            None => Vec::new(),
                        },
                        // TODO: Requires caching messages waa.
                        replies: Vec::new(),
                    })
                }
                GatewayEvent::Pong { data } => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();
                    log::info!(
                        "Received pong! Latency is {}ms",
                        now - u128::from_be_bytes(data.try_into().unwrap())
                    );
                    return;
                }
                GatewayEvent::Ready {} => {
                    log::info!("Ready!");
                    return;
                }
                _ => return,
            };
            let payload = Event {
                platform: "revolt",
                data: payload,
            };

            conn.lock()
                .await
                .publish::<&str, String, ()>(
                    "thang-bridge",
                    serde_json::to_string(&payload).unwrap(),
                )
                .await
                .unwrap();
        });
    }

    Ok(())
}
