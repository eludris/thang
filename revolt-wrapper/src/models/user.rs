use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use super::File;

#[derive(Serialize, Deserialize, Debug, Clone, Default, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialUser"]
#[opt_some_priority]
#[opt_skip_serializing_none]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub avatar: Option<File>,
}

impl ToRedisArgs for User {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let args = serde_json::to_string(self).unwrap();
        args.write_redis_args(out);
    }
}

impl FromRedisValue for User {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let args: String = FromRedisValue::from_redis_value(v)?;
        Ok(serde_json::from_str(&args).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserClear {
    Avatar,
}
