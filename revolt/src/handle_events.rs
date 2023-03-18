use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use models::{Event, Message};
use models::{EventData, Result};
use redis::{aio::Connection, AsyncCommands};
use revolt_wrapper::gateway::Events;
use revolt_wrapper::models::User;
use revolt_wrapper::{Event as GatewayEvent, HttpClient};
use tokio::sync::Mutex;

// TODO: use cache, get channel for server id for nickname from member waa
async fn get_name(author: &str, http: &HttpClient, conn: &Mutex<Connection>) -> Result<String> {
    let user = match conn
        .lock()
        .await
        .get::<String, Option<User>>(format!("users:{}", author))
        .await?
    {
        Some(user) => {
            log::debug!("Got user from cache {:?}", user);
            user
        }
        None => {
            let user = http.fetch_user(author).await?;
            let mut conn = conn.lock().await;
            redis::cmd("HSET")
                .arg(&format!("users:{}", author))
                .arg(&user)
                .query_async(&mut *conn)
                .await?;
            user
        }
    };

    Ok(user.username)
}

async fn handle_event(
    event: GatewayEvent,
    conn: Arc<Mutex<Connection>>,
    channel_id: String,
    bot_id: String,
    http: Arc<HttpClient>,
) -> Result<()> {
    let payload = match event {
        GatewayEvent::Message(msg) => {
            if (msg.author == bot_id && msg.masquerade.is_some()) || msg.channel != channel_id {
                return Ok(());
            }

            EventData::MessageCreate(Message {
                content: msg.content.unwrap_or("".to_string()),
                author: get_name(&msg.author, &http, &conn).await?,
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
            return Ok(());
        }
        GatewayEvent::Ready {} => {
            log::info!("Ready!");
            return Ok(());
        }
        GatewayEvent::ServerMemberUpdate { id, data, clear } => {
            let mut conn = conn.lock().await;
            // To custom use `ToRedisArgs` instead of Vec[(K, V)]
            let key = format!("servers:{}:members:{}", id.server, id.user);

            // Only update if the key exists.
            // As otherwise, there would be a partial cache,
            // without any way to see if a key really doesnt exist or simply hasnt been cached yet.
            if conn.exists::<&str, bool>(&key).await? {
                redis::cmd("HSET")
                    .arg(&key)
                    .arg(&data)
                    .query_async(&mut *conn)
                    .await?;

                // bool is used as a known type, it will always be `None` but I cannot provide a variant.

                let h = clear
                    .into_iter()
                    .map(|c| (format!("{:?}", c), None))
                    .collect::<Vec<(String, Option<bool>)>>();

                conn.hset_multiple::<&str, String, Option<bool>, ()>(&key, h.as_slice())
                    .await?;
            }

            return Ok(());
        }
        GatewayEvent::ServerMemberLeave { id, user } => {
            conn.lock()
                .await
                .del::<&str, ()>(format!("servers:{}:members:{}", id, user).as_str());

            return Ok(());
        }
        GatewayEvent::UserUpdate { id, data, clear } => {
            let mut conn = conn.lock().await;
            let key = format!("users:{}", id);

            if conn.exists::<&str, bool>(&key).await? {
                redis::cmd("HSET")
                    .arg(&key)
                    .arg(&data)
                    .query_async(&mut *conn)
                    .await?;

                let h = clear
                    .into_iter()
                    .map(|c| (format!("{:?}", c), None))
                    .collect::<Vec<(String, Option<bool>)>>();

                conn.hset_multiple::<&str, String, Option<bool>, ()>(&key, h.as_slice())
                    .await?;
            }

            return Ok(());
        }
        _ => return Ok(()),
    };
    let payload = Event {
        platform: "revolt",
        data: payload,
    };

    conn.lock()
        .await
        .publish::<&str, String, ()>("thang-bridge", serde_json::to_string(&payload).unwrap())
        .await
        .unwrap();

    Ok(())
}

pub async fn handle_events(
    events: &mut Events,
    conn: Connection,
    channel_id: String,
    bot_id: String,
    http: Arc<HttpClient>,
) -> Result<()> {
    let conn = Arc::new(Mutex::new(conn));
    while let Some(event) = events.next().await {
        let conn = Arc::clone(&conn);
        let bot_id = bot_id.clone();
        let channel_id = channel_id.clone();
        let http = Arc::clone(&http);
        tokio::spawn(async move { handle_event(event, conn, channel_id, bot_id, http).await });
    }

    Ok(())
}
