use std::sync::Arc;

use models::ThangResult;
use models::{DiscordEvent, Event};
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;
use twilight_gateway::{Event as GatewayEvent, Shard};
use twilight_model::id::{marker::WebhookMarker, Id};

pub async fn handle_events(
    shard: &mut Shard,
    conn: Connection,
    webhook_id: Id<WebhookMarker>,
) -> ThangResult<()> {
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
                GatewayEvent::ChannelPinsUpdate(data) => {
                    Event::Discord(DiscordEvent::ChannelPinsUpdate(data))
                }
                GatewayEvent::ChannelUpdate(data) => {
                    Event::Discord(DiscordEvent::ChannelUpdate(data))
                }
                GatewayEvent::MessageCreate(data) => {
                    // Ignore webhook.
                    if data.author.id.cast::<WebhookMarker>() == webhook_id {
                        return;
                    }

                    Event::Discord(DiscordEvent::MessageCreate(data))
                }
                GatewayEvent::MessageDelete(data) => {
                    Event::Discord(DiscordEvent::MessageDelete(data))
                }
                GatewayEvent::MessageDeleteBulk(data) => {
                    Event::Discord(DiscordEvent::MessageDeleteBulk(data))
                }
                GatewayEvent::MessageUpdate(data) => {
                    Event::Discord(DiscordEvent::MessageUpdate(data))
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
