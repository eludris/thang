use std::sync::Arc;

use eludrs::todel::MessageDisguise;
use eludrs::HttpClient;
use futures::StreamExt;
use models::Config;
use models::Event;
use models::EventData;
use models::Result;
use redis::aio::{MultiplexedConnection, PubSub};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitResponse {
    data: RatelimitData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitData {
    retry_after: u64,
}

pub async fn handle_redis(
    mut pubsub: PubSub,
    conn: MultiplexedConnection,
    client: HttpClient,
    config: Config,
) -> Result<()> {
    let conn = Arc::new(Mutex::new(conn));
    for channel in config {
        if channel.eludris.is_some() {
            pubsub.subscribe(channel.name).await?;
        }
    }

    let mut pubsub = pubsub.into_on_message();

    while let Some(payload) = pubsub.next().await {
        let channel_name = payload.get_channel_name();
        // TODO: handle more of the errors here
        let payload: String = match payload.get_payload() {
            Ok(payload) => payload,
            Err(err) => {
                log::error!("Could not get pubsub payload: {}", err);
                continue;
            }
        };
        let payload: Event = match serde_json::from_str(&payload) {
            Ok(payload) => payload,
            Err(err) => {
                log::error!("Failed to deserialize event payload: {}", err);
                continue;
            }
        };

        let mut conn = conn.lock().await;
        let channel_ids = conn
            .smembers::<_, Option<Vec<u64>>>(format!("eludris:channels:{}", channel_name))
            .await?;

        let channel_ids: Vec<u64> = if let Some(channel_ids) = channel_ids {
            if payload.platform == "eludris" {
                let current_id: u64 = payload.identifier.parse().unwrap();
                channel_ids
                    .into_iter()
                    .filter(|id| id != &current_id)
                    .collect()
            } else {
                channel_ids
            }
        } else {
            log::warn!("No channel id found for channel {}", channel_name);
            continue;
        };

        match payload.data {
            EventData::MessageCreate(msg) => {
                let mut content = msg.content.clone();

                if !msg.replies.is_empty() {
                    let referenced = &msg.replies[0];
                    let mut reply = referenced
                        .content
                        .lines()
                        .map(|l| format!("> {}", l))
                        .collect::<Vec<String>>()
                        .join("\n");
                    let mut name = referenced.author.clone();
                    if name.len() > 32 {
                        name = name.drain(..32).collect();
                    }
                    reply.push_str(&format!("\n@{}", name));
                    content = format!("\n{}\n{}", reply, content);
                }

                let attachments = msg
                    .attachments
                    .iter()
                    .map(|a| format!("![]({})", a))
                    .collect::<Vec<String>>()
                    .join("\n");

                if !attachments.is_empty() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&attachments);
                }

                // Since attachments cause a message to be empty.
                // This should be fine for now, but shouldn't happen in the future.
                if content.is_empty() {
                    continue;
                }

                for channel in channel_ids {
                    client
                        .send_message_with_disguise(
                            channel,
                            &content,
                            MessageDisguise {
                                name: Some(msg.author.clone()),
                                avatar: msg.avatar.clone(),
                            },
                            None,
                        )
                        .await?;
                }
            }
            // Seems unreachable now but is a catchall for future events.
            #[allow(unreachable_patterns)]
            payload => {
                log::warn!("Unhandled payload from pubsub: {:?}", payload)
            }
        }
    }

    Ok(())
}
