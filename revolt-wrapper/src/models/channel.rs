use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "channel_type")]
pub enum Channel {
    TextChannel(TextChannel),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextChannel {
    #[serde(rename = "_id")]
    pub id: String,
    pub server: String,
}

impl ToRedisArgs for TextChannel {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let args = serde_json::to_string(self).unwrap();
        args.write_redis_args(out);
    }
}

impl FromRedisValue for TextChannel {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let args: String = FromRedisValue::from_redis_value(v)?;
        Ok(serde_json::from_str(&args).unwrap())
    }
}
