use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkSummary {
    pub language: Option<String>,
    #[serde(skip)]
    pub provider: Option<String>,
    #[serde(rename(deserialize = "one_sentence_summary"))]
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkMindMap {
    pub language: Option<String>,
    pub mind_map: String,
}
