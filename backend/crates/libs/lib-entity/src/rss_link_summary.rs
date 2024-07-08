use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_link_summary"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize)]
pub struct Model {
    pub link_url: String,
    // 总结版本
    pub version: Option<String>,
    // 总结提供者 (例如 gpt-3.5-turbo)
    pub provider: Option<String>,
    // 总结要点的json字符
    pub summary: Option<String>,
    // 总结语言
    pub language: Option<String>,
    // 关键信息点
    pub key_points: Option<String>,
    // 行动步骤
    pub action_items: Option<String>,
    // 关键词
    pub keywords: Option<String>,
    // 创建时间
    #[serde(skip)]
    #[serde(serialize_with = "to_milli_ts")]
    pub create_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    LinkUrl,
    Version,
    Provider,
    Language,
    Summary,
    KeyPoints,
    ActionItems,
    Keywords,
    CreateAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    LinkUrl,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = String;
    fn auto_increment() -> bool {
        false
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::LinkUrl => ColumnType::String(None).def(),
            Self::Version => ColumnType::String(Some(32u32)).def(),
            Self::Provider => ColumnType::String(Some(32u32)).def(),
            Self::Language => ColumnType::String(Some(8u32)).def(),
            Self::Summary => ColumnType::String(None).def(),
            Self::KeyPoints => ColumnType::String(None).def(),
            Self::ActionItems => ColumnType::String(None).def(),
            Self::Keywords => ColumnType::String(None).def(),
            Self::CreateAt => ColumnType::DateTime
                .def()
                .default(Expr::current_timestamp()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Link,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Link => Entity::belongs_to(super::rss_link::Entity)
                .from(Column::LinkUrl)
                .to(super::rss_link::Column::Link)
                .into(),
        }
    }
}

impl Related<super::rss_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Link.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
