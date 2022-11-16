use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use twilight_model::id::{
    marker::{ChannelMarker, WebhookMarker},
    Id,
};

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
pub type ContextT = Arc<RwLock<Context>>;

pub struct Context {
    pub http: twilight_http::Client,
    pub bridge_webhook_id: Id<WebhookMarker>,
    pub bridge_webhook_token: String,
    pub bridge_channel_id: Id<ChannelMarker>,
    pub redis: redis::Connection,
}
