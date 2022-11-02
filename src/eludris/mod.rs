use crate::types::{ContextT, Message, ThangResult, WsReader};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol::Message as TungstenMessage;
use twilight_http::{api_error::ApiError, error::ErrorType};
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;
use twilight_validate::request::webhook_username;

pub async fn iterate_websocket(mut reader: WsReader, context: ContextT) -> ThangResult<()> {
    tokio::spawn(ping_ws(context.clone()));

    while let Some(message) = reader.next().await {
        if message.as_ref().unwrap().is_text() {
            let data = message.unwrap().into_text().unwrap();
            let msg: Message = serde_json::from_str(&data).unwrap();

            if !msg.author.starts_with("Bridge-") {
                tokio::spawn(loop_webhook(context.clone(), msg));
            }
        }
    }

    Ok(())
}

async fn loop_webhook(context: ContextT, msg: Message) -> ThangResult<()> {
    let username = match webhook_username(&msg.author) {
        Ok(_) => &msg.author,
        Err(_) => "Eludris Bridge",
    };

    let length = msg.content.chars().count();

    let content = if msg.content.chars().count() == 0 {
        return Ok(());
    } else if length <= MESSAGE_CONTENT_LENGTH_MAX {
        msg.content
    } else {
        format!("{}...", &msg.content[..MESSAGE_CONTENT_LENGTH_MAX - 3])
    };

    loop {
        match context
            .http
            .execute_webhook(context.bridge_webhook_id, &context.bridge_webhook_token)
            .content(&content)
            .unwrap()
            .username(username)
            .unwrap()
            .exec()
            .await
        {
            Ok(_) => return Ok(()),
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

async fn ping_ws(context: ContextT) -> ThangResult<()> {
    loop {
        context
            .eludris_ws_writer
            .lock()
            .await
            .send(TungstenMessage::Ping(b"woo".to_vec()))
            .await?;

        time::sleep(Duration::from_secs(15)).await;
    }
}
