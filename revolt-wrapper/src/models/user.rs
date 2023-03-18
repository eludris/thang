use redis;
use redis_derive::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Default, OptionalStruct, ToRedisArgs, FromRedisValue,
)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone, ToRedisArgs)]
#[optional_name = "PartialUser"]
#[opt_some_priority]
#[opt_skip_serializing_none]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserClear {
    Avatar,
}
