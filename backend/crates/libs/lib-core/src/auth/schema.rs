use crate::error::ErrorInService;
use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use lib_entity::rss_account::{Gender, Model as AccountModelInDB};
use sea_orm::FromQueryResult;

// 用于传递给外部的整理过的订阅源数据
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
#[builder(name = "AccountModelBuilder")]
#[builder(build_fn(error = "ErrorInService"))]
pub struct AccountModel {
    #[serde(skip)]
    pub id: i64, // 主键
    pub nick_name: Option<String>,    // 昵称
    pub email: Option<String>,        // 邮箱
    pub birth: Option<NaiveDateTime>, // 出生日期
    pub gender: Option<Gender>,       // 性别 1 男 2 女
}

impl FromQueryResult for AccountModel {
    fn from_query_result(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Self, sea_orm::prelude::DbErr> {
        let mut model_builder = AccountModelBuilder::default();
        let model = model_builder
            .id(res.try_get(pre, "id")?)
            .nick_name(res.try_get(pre, "nick_name").unwrap_or(None))
            .email(res.try_get(pre, "email").unwrap_or(None))
            .birth(res.try_get(pre, "birth").unwrap_or(None))
            .gender(res.try_get(pre, "gender").unwrap_or(None))
            .build()
            .map_err(|e| {
                sea_orm::prelude::DbErr::Custom(format!("AccountModelBuilder build error: {:?}", e))
            })?;
        Ok(model)
    }

    fn from_query_result_optional(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Option<Self>, sea_orm::prelude::DbErr> {
        if let Ok(model) = AccountModel::from_query_result(res, pre) {
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

impl From<AccountModelInDB> for AccountModel {
    fn from(value: lib_entity::rss_account::Model) -> Self {
        AccountModel {
            id: value.id,
            nick_name: value.nick_name,
            email: value.email,
            birth: value.birth,
            gender: value.gender,
        }
    }
}

// 构建查询用户的请求
#[derive(Debug, Default)]
pub struct QueryAccountByIDRequest {
    pub id: i64,
}

// 构建注册用户的请求
#[derive(Debug, Deserialize)]
pub struct RegisterAccountRequest {
    // 用户邮箱
    pub email: String,
    pub nick_name: Option<String>,
    pub password: String,
}

// 登录用户的请求
#[derive(Debug, Deserialize)]
pub struct LoginAccountRequest {
    // 用户邮箱
    pub email: String,
    pub password: String,
}

// 登录请求的响应
#[derive(Debug, Serialize)]
pub struct LoginAccountResponse {
    pub token: String,
    pub account: AccountModel,
}

// 更新用户信息的请求
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: i64,
    pub nick_name: Option<String>, // 昵称
}
