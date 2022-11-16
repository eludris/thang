use std::time::Duration;

use crate::types::ThangResult;
use futures::StreamExt;
use models::Event;
use redis::aio::Connection;
use todel::models::Payload;
use tokio::time;
use twilight_http::{api_error::ApiError, error::ErrorType, Client};
use twilight_model::channel::Webhook;
use twilight_validate::{message::MESSAGE_CONTENT_LENGTH_MAX, request::webhook_username};

pub async fn handle_redis(conn: Connection, http: Client, webhook: Webhook) -> ThangResult<()> {
    let mut pubsub = conn.into_pubsub();

    pubsub.subscribe("thang-bridge").await?;
    let mut pubsub = pubsub.into_on_message();

    while let Some(payload) = pubsub.next().await {
        // TODO: handle more of the errors here
        let payload: Event =
            serde_json::from_str(&payload.get_payload::<String>().unwrap()).unwrap();
        match payload {
            Event::Eludris(Payload::Message(msg)) => {
                if !msg.author.starts_with("Bridge-") {
                    let content = if msg.content.len() > MESSAGE_CONTENT_LENGTH_MAX {
                        format!(
                            "{}... truncated message",
                            &msg.content[..MESSAGE_CONTENT_LENGTH_MAX - 21]
                        )
                    } else {
                        msg.content
                    };
                    loop {
                        let token = webhook.token.as_ref().unwrap();
                        let mut req = http
                            .execute_webhook(webhook.id, token)
                            .content(&content)
                            .unwrap();
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
                                    time::sleep(Duration::from_secs_f64(ratelimit.retry_after))
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
            Event::Discord(_) => {}
            payload => {
                log::info!("Unhandled payload from pubsub: {:?}", payload)
            }
        }
    }

    Ok(())
}
