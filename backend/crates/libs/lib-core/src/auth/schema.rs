use crate::error::ErrorInService;
use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use sea_orm::FromQueryResult;

// 用于传递给外部的整理过的订阅源数据
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
#[builder(build_fn(error = "ErrorInService"))]
pub struct CustomerModel {
    #[serde(skip)]
    pub id: i64, // 主键
    pub full_name: Option<String>,    // 全名
    pub nick_name: Option<String>,    // 昵称
    pub email: Option<String>,        // 邮箱
    pub mobile: Option<String>,       // 手机号
    pub password: Option<String>,     // 密码
    pub avatar: Option<String>,       // 头像
    pub birth: Option<NaiveDateTime>, // 出生日期
    pub gender: Option<i32>,          // 性别 1 男 2 女
    pub sign: Option<String>,         // 个性签名
    pub region: Option<String>,       // 地区
    pub ai_token_amount: Option<i64>, // AI Token 数量
    #[serde(serialize_with = "to_milli_tsopt")]
    pub last_login_time: Option<NaiveDateTime>, // 上一次登录时间
    pub apple_id: Option<String>,     // 苹果的用户标识
    pub google_id: Option<String>,    // 谷歌用户标识
}

impl FromQueryResult for CustomerModel {
    fn from_query_result(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Self, sea_orm::prelude::DbErr> {
        let mut model_builder = CustomerModelBuilder::default();
        let model = model_builder
            .id(res.try_get(pre, "id")?)
            .full_name(res.try_get(pre, "full_name").unwrap_or(None))
            .nick_name(res.try_get(pre, "nick_name").unwrap_or(None))
            .email(res.try_get(pre, "email").unwrap_or(None))
            .mobile(res.try_get(pre, "mobile").unwrap_or(None))
            .password(res.try_get(pre, "password").unwrap_or(None))
            .avatar(res.try_get(pre, "avatar").unwrap_or(None))
            .birth(res.try_get(pre, "birth").unwrap_or(None))
            .gender(res.try_get(pre, "gender").unwrap_or(None))
            .sign(res.try_get(pre, "sign").unwrap_or(None))
            .region(res.try_get(pre, "region").unwrap_or(None))
            .ai_token_amount(res.try_get(pre, "ai_token_amount").unwrap_or(None))
            .last_login_time(res.try_get(pre, "last_login_time").unwrap_or(None))
            .apple_id(res.try_get(pre, "apple_id").unwrap_or(None))
            .google_id(res.try_get(pre, "google_id").unwrap_or(None))
            .build()
            .map_err(|e| {
                sea_orm::prelude::DbErr::Custom(format!(
                    "CustomerModelBuilder build error: {:?}",
                    e
                ))
            })?;
        Ok(model)
    }

    fn from_query_result_optional(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Option<Self>, sea_orm::prelude::DbErr> {
        if let Ok(model) = CustomerModel::from_query_result(res, pre) {
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

// 构建查询用户的请求
#[derive(Debug, Default)]
pub struct QueryCustomerByIDRequest {
    pub id: i64,
}
