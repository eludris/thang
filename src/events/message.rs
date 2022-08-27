use crate::types::ThangResult;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn on_message(_msg: MessageCreate, _http: Arc<Client>) -> ThangResult<()> {
    Ok(())
}
