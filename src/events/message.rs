use crate::types::{Context, ThangResult};
use std::sync::Arc;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn on_message(_msg: MessageCreate, _context: Arc<Context>) -> ThangResult<()> {
    Ok(())
}
