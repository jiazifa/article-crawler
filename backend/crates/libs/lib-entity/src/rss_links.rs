use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_links"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    #[serde(skip)]
    pub id: i64,
    // 标题
    pub title: String,
    // 链接
    pub link: String,
    // 描述(可能包含 html)
    pub description: Option<String>,
    // 纯文本描述
    pub desc_pure_txt: Option<String>,
    // 图片 , 用于显示链接的图片
    pub images: Option<Json>,
    // 作者
    pub authors: Option<Json>,
    // tags array of string
    pub tags: Option<Json>,
    // 发布时间
    #[serde(serialize_with = "to_milli_tsopt")]
    pub published_at: Option<NaiveDateTime>,
    // 创建时间
    #[serde(skip)]
    #[serde(serialize_with = "to_milli_ts")]
    pub created_at: NaiveDateTime,
    // 更新时间
    #[serde(skip)]
    #[serde(serialize_with = "to_milli_ts")]
    pub updated_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Title,
    Link,
    Description,
    DescPureTxt,
    Images,
    Authors,
    Tags,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        true
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Title => ColumnType::String(Some(255u32)).def().nullable(),
            Self::Description => ColumnType::Text.def().nullable(),
            Self::DescPureTxt => ColumnType::Text.def().nullable(),
            Self::Images => ColumnType::Array(RcOrArc::new(ColumnType::Text))
                .def()
                .nullable(),
            Self::Authors => ColumnType::Text.def().nullable(),
            Self::Tags => ColumnType::Array(RcOrArc::new(ColumnType::String(Some(64u32))))
                .def()
                .nullable(),
            Self::Link => ColumnType::Text.def(),
            Self::PublishedAt => ColumnType::DateTime.def().nullable(),
            Self::CreatedAt => ColumnType::DateTime
                .def()
                .default(Expr::current_timestamp()),
            Self::UpdatedAt => ColumnType::DateTime
                .def()
                .default(Expr::current_timestamp()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Subscrption,
    Summary,
    MindMap,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Subscrption => Entity::has_many(super::rss_subscriptions::Entity).into(),
            Self::Summary => Entity::has_one(super::rss_link_summary::Entity).into(),
            Self::MindMap => Entity::has_one(super::rss_link_mindmap::Entity).into(),
        }
    }
}

impl Related<super::rss_subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        super::rss_links_subscriptions::Relation::Subscription.def()
    }
}

impl Related<super::rss_link_summary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Summary.def()
    }
}

impl Related<super::rss_link_mindmap::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MindMap.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
