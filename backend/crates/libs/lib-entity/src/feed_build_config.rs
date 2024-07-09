use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Subscription Type
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(32))")]
pub enum SourceType {
    #[sea_orm(string_value = "Unknown")]
    Unknown,
    // Rss
    #[sea_orm(string_value = "Rss")]
    Rss,
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "feed_build_config"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

// 频率的定义: 一般来说，频率是一个浮点数，表示多少分钟更新一次 例如 60.0 表示一小时更新一次 30.0 表示半小时更新一次
#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Serialize)]
pub struct Model {
    // 订阅源Id
    pub subscription_id: i64,
    // 初始频率
    pub initial_frequency: f32,
    // 自适应 频率
    pub fitted_frequency: Option<f32>,
    // 是否启用自适应
    pub fitted_adaptive: bool,
    // 订阅源类型(决定了更新的方式)
    pub source_type: SourceType,
    // 最近一次更新时间
    #[serde(serialize_with = "to_milli_tsopt")]
    pub last_build_at: Option<NaiveDateTime>,
}

impl Model {
    pub fn get_frequency(&self) -> f32 {
        self.fitted_frequency.unwrap_or(self.initial_frequency)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    SubscriptionId,
    InitialFrequency,
    FittedFrequency,
    FittedAdaptive,
    SourceType,
    LastBuildAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    SubscriptionId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        false
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::SubscriptionId => ColumnType::Integer.def(),
            Self::InitialFrequency => ColumnType::Float.def(),
            Self::FittedFrequency => ColumnType::Float.def().nullable(),
            Self::FittedAdaptive => ColumnType::Boolean.def(),
            Self::SourceType => ColumnType::SmallInteger.def(),
            Self::LastBuildAt => ColumnType::DateTime.def().nullable(),
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
