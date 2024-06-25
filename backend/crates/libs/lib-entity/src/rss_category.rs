use chrono::naive::serde::ts_milliseconds::serialize as to_milli_ts;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_category"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize)]
pub struct Model {
    #[serde(skip)]
    pub id: i64,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    // 父节点 Id
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    // 排序序列
    pub sort_order: Option<i64>,
    #[serde(serialize_with = "to_milli_ts")]
    pub created_at: NaiveDateTime,
    #[serde(serialize_with = "to_milli_ts")]
    pub updated_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Identifier,
    Title,
    Description,
    ParentId,
    Icon,
    SortOrder,
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
            Self::Title => ColumnType::String(Some(32u32)).def().null(),
            Self::Description => ColumnType::String(Some(255u32)).def().null(),
            Self::ParentId => ColumnType::Integer.def().null(),
            Self::Icon => ColumnType::String(Some(255u32)).def().null(),
            Self::SortOrder => ColumnType::Integer.def().null(),
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
    Subscriptions,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Subscriptions => Entity::has_many(super::rss_subscriptions::Entity).into(),
        }
    }
}

impl Related<super::rss_subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriptions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
