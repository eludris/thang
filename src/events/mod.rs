mod message;
mod shard;

use crate::types::ThangResult;
use log::error;
use message::on_message;
use shard::on_shard_connect;
use std::{future::Future, sync::Arc};
use twilight_gateway::Event;
use twilight_http::Client;

pub async fn handle_event(shard_id: u64, event: Event, http: Arc<Client>) {
    match event {
        Event::ShardConnected(_) => on_shard_connect(shard_id),
        Event::MessageCreate(msg) => async_wrapper(on_message(*msg, http), "message_create"),
        _ => {}
    };
}

fn async_wrapper(todo: impl Future<Output = ThangResult<()>> + Send + 'static, name: &'static str) {
    tokio::spawn(async move {
        if let Err(e) = todo.await {
            error!("Failed to execute {} handler: {}", name, e);
        }
    });
}
