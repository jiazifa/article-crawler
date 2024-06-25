use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i64>,
    pub children: Option<Vec<Category>>,
}
