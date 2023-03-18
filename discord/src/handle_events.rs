use std::sync::Arc;

use models::{Event, EventData, Message, Reply, Result};
use redis::{aio::Connection, AsyncCommands};
use tokio::sync::Mutex;
use twilight_gateway::{Event as GatewayEvent, Shard};
use twilight_model::id::{
    marker::{ChannelMarker, WebhookMarker},
    Id,
};

fn get_user_avatar(msg: &twilight_model::channel::Message) -> String {
    if let Some(avatar) = msg.author.avatar {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            msg.author.id, avatar
        )
    } else {
        format!(
            "https://cdn.discordapp.com/embed/avatars/{}.png",
            msg.author.discriminator % 5
        )
    }
}

fn get_avatar(msg: &twilight_model::channel::Message) -> String {
    match &msg.member {
        Some(member) => match member.avatar {
            Some(avatar) => format!(
                "https://cdn.discordapp.com/guilds/{}/users/{}/avatars/{}.png",
                msg.guild_id.unwrap(),
                msg.author.id,
                avatar
            ),
            None => get_user_avatar(msg),
        },
        None => get_user_avatar(msg),
    }
}

fn get_name(msg: &twilight_model::channel::Message) -> String {
    match &msg.member {
        Some(member) => match &member.nick {
            Some(nick) => nick.clone(),
            None => msg.author.name.clone(),
        },
        None => msg.author.name.clone(),
    }
}

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
                            author: get_name(&data),
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
                            avatar: Some(get_avatar(&data)),
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
