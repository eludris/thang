use std::sync::Arc;

use models::{Event, EventData, Message, Reply, Result};
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;
use twilight_gateway::{Event as GatewayEvent, Shard};
use twilight_model::id::{
    marker::{ChannelMarker, WebhookMarker},
    Id,
};

pub async fn handle_events(
    shard: &mut Shard,
    conn: Connection,
    webhook_id: Id<WebhookMarker>,
    channel_id: Id<ChannelMarker>,
) -> Result<()> {
    let conn = Arc::new(Mutex::new(conn));
    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                log::warn!("error receiving event");

                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };

        let conn = Arc::clone(&conn);
        tokio::spawn(async move {
            let payload = match event {
                GatewayEvent::MessageCreate(data) => {
                    // Ignore webhook.
                    if data.author.id.cast::<WebhookMarker>() == webhook_id
                        || data.channel_id != channel_id
                    {
                        return;
                    }

                    Event {
                        platform: "discord",
                        data: EventData::MessageCreate(Message {
                            content: data.content.clone(),
                            author: data.author.name.clone(),
                            attachments: data
                                .attachments
                                .clone()
                                .into_iter()
                                .map(|a| a.url)
                                .collect(),
                            replies: match &data.referenced_message {
                                Some(msg) => vec![Reply {
                                    content: msg.content.clone(),
                                    author: msg.author.name.clone(),
                                }],
                                None => Vec::new(),
                            },
                        }),
                    }
                }
                _ => return,
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
