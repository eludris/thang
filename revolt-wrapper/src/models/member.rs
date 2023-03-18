use redis;
use redis_derive::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, ToRedisArgs, FromRedisValue)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Default, OptionalStruct, ToRedisArgs, FromRedisValue,
)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone, ToRedisArgs)]
#[optional_name = "PartialMember"]
#[opt_some_priority]
#[opt_skip_serializing_none]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToRedisArgs)]
pub enum MemberClear {
    Nickname,
}
