use crate::types::ThangResult;
use log::info;

pub fn on_shard_connect(shard_id: u64) -> ThangResult<()> {
    info!("Connected on shard {}", shard_id);
    Ok(())
}
