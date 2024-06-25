use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_subscriptions"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Serialize, Deserialize)]
pub struct Model {
    #[serde(skip)]
    pub id: i64,
    // 唯一标识
    pub identifier: String,
    // 标题
    pub title: String,
    // 描述
    pub description: Option<String>,
    // Rss 链接地址
    pub link: Option<String>,
    // 对应rss链接提供方的网站
    pub site_link: Option<String>,
    // 分类Id
    pub category_id: Option<i64>,
    // logo URL
    pub logo: Option<String>,
    // 语言
    pub language: Option<String>,
    // 评分(score)
    pub rating: Option<i32>,
    // 文章左上角的小icon
    pub visual_url: Option<String>,
    // 排序序列
    pub sort_order: Option<i32>,
    // 发布日期
    #[serde(serialize_with = "to_milli_tsopt")]
    pub pub_date: Option<NaiveDateTime>,
    // 最近更新时间
    #[serde(serialize_with = "to_milli_tsopt")]
    pub last_build_date: Option<NaiveDateTime>,
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
    Identifier,
    Title,
    Description,
    Link,
    SiteLink,
    CategoryId,
    Logo,
    Language,
    Rating,
    VisualUrl,
    SortOrder,
    PubDate,
    LastBuildDate,
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
            Self::Identifier => ColumnType::String(Some(32u32)).def(),
            Self::Title => ColumnType::String(Some(255u32)).def().nullable(),
            Self::Description => ColumnType::Text.def().nullable(),
            Self::Link => ColumnType::Text.def().nullable(),
            Self::CategoryId => ColumnType::Integer.def().nullable(),
            Self::SiteLink => ColumnType::String(Some(255u32)).def().nullable(),
            Self::Logo => ColumnType::Text.def().nullable(),
            Self::Language => ColumnType::String(Some(64u32)).def().nullable(),
            Self::Rating => ColumnType::Integer.def().nullable(),
            Self::VisualUrl => ColumnType::Text.def().nullable(),
            Self::SortOrder => ColumnType::Integer.def().nullable(),
            Self::PubDate => ColumnType::DateTime.def().nullable(),
            Self::LastBuildDate => ColumnType::DateTime.def().nullable(),
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
    Category,
    Links,
    SubscriptorOffset,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Category => Entity::belongs_to(super::rss_category::Entity)
                .from(Column::CategoryId)
                .to(super::rss_category::Column::Id)
                .into(),
            Self::Links => Entity::has_many(super::rss_links::Entity).into(),
            Self::SubscriptorOffset => {
                Entity::has_one(super::rss_subscrip_count_offset::Entity).into()
            }
        }
    }
}

impl Related<super::rss_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::rss_links::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Links.def()
    }
}

impl Related<super::rss_subscrip_count_offset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptorOffset.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
