use chrono::naive::serde::ts_milliseconds::serialize as to_milli_ts;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_subscription_update_config"
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
    pub adaptive: bool,
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
    Adaptive,
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
            Self::Adaptive => ColumnType::Boolean.def(),
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
            Self::Subscription => Entity::belongs_to(super::rss_subscriptions::Entity)
                .from(Column::SubscriptionId)
                .to(super::rss_subscriptions::Column::Id)
                .into(),
        }
    }
}

impl Related<super::rss_subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
