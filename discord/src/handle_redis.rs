use std::time::Duration;

use futures::StreamExt;
use lazy_static::lazy_static;
use models::Event;
use models::EventData;
use models::Result;
use redis::aio::Connection;
use regex::Regex;
use tokio::time;
use twilight_http::{api_error::ApiError, error::ErrorType, Client};
use twilight_mention::Mention;
use twilight_model::channel::Webhook;
use twilight_validate::{message::MESSAGE_CONTENT_LENGTH_MAX, request::webhook_username};

pub async fn handle_redis(conn: Connection, http: Client, webhook: Webhook) -> Result<()> {
    lazy_static! {
        static ref EMOJI_REGEX: Regex = Regex::new(r":(\w+):").unwrap();
    }
    let mut pubsub = conn.into_pubsub();

    pubsub.subscribe("thang-bridge").await?;
    let mut pubsub = pubsub.into_on_message();

    while let Some(payload) = pubsub.next().await {
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

        if payload.platform == "discord" {
            continue;
        }

        match payload.data {
            EventData::MessageCreate(msg) => {
                let emojis = http
                    .emojis(webhook.guild_id.unwrap())
                    .await
                    .unwrap()
                    .models()
                    .await
                    .unwrap();

                let mut content = msg.content;
                EMOJI_REGEX.captures_iter(&content.clone()).for_each(|c| {
                    let found = c.get(0).unwrap().as_str();
                    let name = c.get(1).unwrap().as_str();
                    if let Some(emoji) = emojis.iter().find(|e| e.name == name) {
                        content = content.replace(found, &emoji.mention().to_string());
                    }
                });

                let content = if content.len() > MESSAGE_CONTENT_LENGTH_MAX {
                    format!(
                        "{}... truncated message",
                        &content[..MESSAGE_CONTENT_LENGTH_MAX - 21]
                    )
                } else {
                    content
                };
                loop {
                    let token = webhook.token.as_ref().unwrap();
                    let mut req = match http.execute_webhook(webhook.id, token).content(&content) {
                        Ok(req) => req,
                        Err(err) => {
                            log::error!("Failed to create webhook request: {}", err);
                            break;
                        }
                    };

                    if let Some(avatar) = &msg.avatar {
                        req = req.avatar_url(avatar);
                    }

                    if webhook_username(&msg.author).is_ok() {
                        req = req.username(&msg.author).unwrap();
                    }
                    match req.await {
                        Ok(_) => break,
                        Err(err) => {
                            if let ErrorType::Response {
                                body: _,
                                error: ApiError::Ratelimited(ratelimit),
                                status: _,
                            } = err.kind()
                            {
                                log::warn!(
                                    "Bridge webhook is ratelimited, waiting {}s",
                                    ratelimit.retry_after
                                );
                                time::sleep(Duration::from_secs_f64(ratelimit.retry_after)).await;
                            }
                        }
                    }
                }
            }
            // Unreachable now but not for new events.
            #[allow(unreachable_patterns)]
            payload => {
                log::warn!("Unhandled payload from pubsub: {:?}", payload)
            }
        }
    }

    Ok(())
}
