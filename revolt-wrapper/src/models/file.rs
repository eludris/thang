use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    pub tag: String,
    pub filename: String,
}

impl File {
    pub fn url(&self) -> String {
        format!(
            "https://autumn.revolt.chat/{}/{}/{}",
            self.tag, self.id, self.filename
        )
    }
}
