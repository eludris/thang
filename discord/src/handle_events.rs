use std::sync::Arc;

use futures::StreamExt;
use models::ThangResult;
use models::{DiscordEvent, Event};
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;
use twilight_gateway::{cluster::Events, Event as GatewayEvent};

pub async fn handle_events(mut events: Events, conn: Connection) -> ThangResult<()> {
    let conn = Arc::new(Mutex::new(conn));
    while let Some((_, event)) = events.next().await {
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
