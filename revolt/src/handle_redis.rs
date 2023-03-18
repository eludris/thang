use std::sync::Arc;

use futures::StreamExt;
use models::Event;
use models::EventData;
use models::Result;
use redis::aio::Connection;
use revolt_wrapper::models::Masquerade;
use revolt_wrapper::HttpClient;

pub async fn handle_redis(redis: Connection, rest: Arc<HttpClient>, channel: String) -> Result<()> {
    let mut pubsub = redis.into_pubsub();

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

        if payload.platform == "revolt" {
            continue;
        }

        match payload.data {
            EventData::MessageCreate(msg) => {
                let /*mut*/ content = msg.content.clone();

                // if !msg.replies.is_empty() {
                //     let referenced = &msg.replies[0];
                //     let mut reply = referenced
                //         .content
                //         .lines()
                //         .map(|l| format!("> {}", l))
                //         .collect::<Vec<String>>()
                //         .join("\n");
                //     let mut name = referenced.author.clone();
                //     if name.len() > 32 {
                //         name = name.drain(..32).collect();
                //     }
                //     reply.push_str(&format!("\n@{}", name));
                //     content = format!("\n{}\n{}", reply, content);
                // }

                // let attachments = msg
                //     .attachments
                //     .iter()
                //     .map(|a| a.as_ref())
                //     .collect::<Vec<&str>>()
                //     .join("\n");

                // if !attachments.is_empty() {
                //     if !content.is_empty() {
                //         content.push('\n');
                //     }
                //     content.push_str(&attachments);
                // }

                rest.send_message(&channel)
                    .content(content)
                    .masquerade(Masquerade {
                        name: Some(msg.author),
                        avatar: None,
                    })
                    .send()
                    .await?;
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
