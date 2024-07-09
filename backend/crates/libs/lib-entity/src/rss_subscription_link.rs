use chrono::naive::serde::ts_milliseconds::serialize as to_milli_ts;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_subscription_link"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Serialize)]
pub struct Model {
    pub id: i64,
    // 订阅源Id
    pub subscription_id: i64,
    // links id
    pub link_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    SubscriptionId,
    LinkId,
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
            Self::SubscriptionId => ColumnType::Integer.def(),
            Self::LinkId => ColumnType::Integer.def(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Subscription,
    Link,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Subscription => Entity::belongs_to(super::rss_subscription::Entity)
                .from(Column::SubscriptionId)
                .to(super::rss_subscription::Column::Id)
                .into(),
            Self::Link => Entity::belongs_to(super::feed_link::Entity)
                .from(Column::LinkId)
                .to(super::feed_link::Column::Id)
                .into(),
        }
    }
}

impl Related<super::rss_subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::feed_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Link.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
