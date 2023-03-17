use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialMember"]
#[opt_some_priority]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberClear {
    Nickname,
    Avatar,
}
