use log::info;

pub fn on_shard_connect(shard_id: u64) {
    info!("Connected on shard {}", shard_id);
}
