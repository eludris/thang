use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialUser"]
#[opt_some_priority]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserClear {
    Avatar,
}
