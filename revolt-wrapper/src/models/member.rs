use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub nickname: Option<String>,
}
