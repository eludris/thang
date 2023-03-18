use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

impl ToRedisArgs for MemberCompositeKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let args = vec![self.server.clone(), self.user.clone()];
        args.write_redis_args(out);
    }
}

impl FromRedisValue for MemberCompositeKey {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let args: Vec<String> = FromRedisValue::from_redis_value(v)?;
        Ok(MemberCompositeKey {
            server: args[0].clone(),
            user: args[1].clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialMember"]
#[opt_some_priority]
#[opt_skip_serializing_none]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub nickname: Option<String>,
}

impl ToRedisArgs for Member {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let args = serde_json::to_string(self).unwrap();
        args.write_redis_args(out);
    }
}

impl FromRedisValue for Member {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let args: String = FromRedisValue::from_redis_value(v)?;
        Ok(serde_json::from_str(&args).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberClear {
    Nickname,
}
