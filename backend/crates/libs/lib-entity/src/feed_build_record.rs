use chrono::naive::serde::ts_milliseconds::serialize as to_milli_ts;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// define a enum Entity
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Status {
    #[sea_orm(num_value = 0)]
    Unknow,
    #[sea_orm(num_value = 1)]
    Faild,
    #[sea_orm(num_value = 10)]
    FweSuccess,
    #[sea_orm(num_value = 20)]
    MostSuccess,
    #[sea_orm(num_value = 99)]
    Success,
}

// impl for default
impl Default for Status {
    fn default() -> Self {
        Self::Unknow
    }
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "feed_build_record"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize)]
pub struct Model {
    // 主键Id
    pub identifier: String,
    // 订阅源Id
    pub subscription_id: i64,
    // 状态 表示订阅源此次更新的状态， 成功 / 失败 / 其他 如果是失败，需要记录失败原因
    pub status: Status,
    // 备注 失败原因
    pub remark: String,
    #[serde(serialize_with = "to_milli_ts")]
    pub created_at: NaiveDateTime,
}

impl Model {
    pub fn is_success(&self) -> bool {
        match self.status {
            Status::Unknow => false,
            Status::Faild => false,
            Status::FweSuccess => false,
            Status::MostSuccess => true,
            Status::Success => true,
        }
    }

    pub fn is_failed(&self) -> bool {
        match self.status {
            Status::Unknow => false,
            Status::Faild => true,
            Status::FweSuccess => false,
            Status::MostSuccess => false,
            Status::Success => false,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Identifier,
    SubscriptionId,
    Status,
    Remark,
    CreatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Identifier,
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
            Self::Identifier => ColumnType::String(Some(32u32)).def(),
            Self::SubscriptionId => ColumnType::Integer.def(),
            Self::Status => ColumnType::SmallInteger.def(),
            Self::Remark => ColumnType::String(Some(255u32)).def().nullable(),
            Self::CreatedAt => ColumnType::Timestamp.def(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Subscription,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Subscription => Entity::belongs_to(super::feed_subscription::Entity)
                .from(Column::SubscriptionId)
                .to(super::feed_subscription::Column::Id)
                .into(),
        }
    }
}

impl Related<super::feed_subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
