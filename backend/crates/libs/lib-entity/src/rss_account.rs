use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Gender
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Gender {
    #[sea_orm(num_value = 0)]
    MALE,
    #[sea_orm(num_value = 1)]
    FEMALE,
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "rss_account"
    }
    fn schema_name(&self) -> Option<&str> {
        // Some("dasv")
        None
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Deserialize)]
pub struct Model {
    #[serde(skip)]
    pub id: i64, // 主键
    pub nick_name: Option<String>,    // 昵称
    pub email: Option<String>,        // 邮箱
    pub password: Option<String>,     // 密码
    pub avatar: Option<String>,       // 头像
    pub birth: Option<NaiveDateTime>, // 出生日期
    pub gender: Option<Gender>,       // 性别 1 男 2 女
    #[serde(skip)]
    #[serde(serialize_with = "to_milli_ts")]
    pub create_time: NaiveDateTime, // 创建时间（注册时间）
    #[serde(skip)]
    #[serde(serialize_with = "to_milli_ts")]
    pub update_time: NaiveDateTime, // 更新时间
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    NickName,
    Email,
    Password,
    Avatar,
    Birth,
    Gender,
    LastLoginTime,
    CreateTime,
    UpdateTime,
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
            Self::NickName => ColumnType::String(Some(100)).def().nullable(),
            Self::Email => ColumnType::String(Some(100)).def().nullable(),
            Self::Password => ColumnType::String(Some(40)).def().nullable(),
            Self::Avatar => ColumnType::Binary(BlobSize::Medium).def().nullable(),
            Self::Birth => ColumnType::Date.def().nullable(),
            Self::Gender => ColumnType::SmallInteger.def().nullable(),
            Self::LastLoginTime => ColumnType::DateTime.def().nullable(),

            Self::CreateTime => ColumnType::DateTime
                .def()
                .default(Expr::current_timestamp()),
            Self::UpdateTime => ColumnType::DateTime
                .def()
                .default(Expr::current_timestamp()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Token,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Token => Entity::has_one(super::rss_account_token::Entity).into(),
        }
    }
}

impl Related<super::rss_account_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Token.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
