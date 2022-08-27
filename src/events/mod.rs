mod message;
mod shard;

use crate::types::ThangResult;
use message::on_message;
use shard::on_shard_connect;
use std::sync::Arc;
use twilight_gateway::Event;
use twilight_http::Client;

pub async fn handle_event(shard_id: u64, event: Event, http: Arc<Client>) -> ThangResult<()> {
    return match event {
        Event::MessageCreate(msg) => on_message(*msg, http).await,
        Event::ShardConnected(_) => on_shard_connect(shard_id),
        _ => Ok(()),
    };
}
