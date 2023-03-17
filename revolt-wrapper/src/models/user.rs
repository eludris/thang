use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
}
