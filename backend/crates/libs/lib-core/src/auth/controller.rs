use lib_entity::rss_account;

use crate::{error::ErrorInService, DBConnection};

use super::schema::{
    AccountModel, LoginAccountRequest, LoginAccountResponse, QueryAccountByIDRequest,
    RegisterAccountRequest,
};
use sea_orm::{entity::*, query::*};

impl QueryAccountByIDRequest {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

pub struct AccountController;

impl AccountController {
    pub async fn register_account(
        &self,
        req: RegisterAccountRequest,
        conn: &DBConnection,
    ) -> Result<AccountModel, ErrorInService> {
        let query =
            rss_account::Entity::find().filter(rss_account::Column::Email.eq(&req.email.clone()));
        let account = query.one(conn).await.map_err(ErrorInService::DBError)?;
        if account.is_some() {
            return Err(ErrorInService::Custom("account already exists".to_string()));
        }
        // hashed password
        let hashed_password_dist = md5::compute(req.password.as_bytes());
        let hashed_password = format!("{:x}", hashed_password_dist);
        // add new account
        let new_account = rss_account::ActiveModel {
            email: Set(Some(req.email)),
            nick_name: Set(req.nick_name.clone()),
            password: Set(Some(hashed_password)),
            ..Default::default()
        };
        let updated = new_account.insert(conn).await?;
        Ok(updated.into())
    }

    // login account
    pub async fn login_account(
        &self,
        req: LoginAccountRequest,
        conn: &DBConnection,
    ) -> Result<LoginAccountResponse, ErrorInService> {
        // query if account exists
        let query = rss_account::Entity::find().filter(rss_account::Column::Email.eq(&req.email));
        let account = query.one(conn).await.map_err(ErrorInService::DBError)?;
        let account = account.ok_or(ErrorInService::Custom("account not found".to_string()))?;
        // check password
        let hashed_password_dist = md5::compute(req.password.as_bytes());
        let hashed_password = format!("{:x}", hashed_password_dist);
        if account.password != Some(hashed_password) {
            return Err(ErrorInService::Custom("password error".to_string()));
        }
        // create new token, rule: email + password + salt => md5
        let token_origin = format!("{}{}{}", req.email, req.password, "salt");
        let token_dist = md5::compute(token_origin.as_bytes());
        let token = format!("{:x}", token_dist);

        Ok(LoginAccountResponse {
            token,
            account: account.into(),
        })
    }

    pub async fn account_info(
        &self,
        account_id: i64,
        conn: &DBConnection,
    ) -> Result<Option<AccountModel>, ErrorInService> {
        let model = rss_account::Entity::find()
            .filter(rss_account::Column::Id.eq(account_id))
            .one(conn)
            .await
            .map_err(ErrorInService::DBError)?;
        let account: Option<AccountModel> = model.map(|m| m.into());
        Ok(account)
    }

    pub async fn query_account_by_id(
        &self,
        req: QueryAccountByIDRequest,
        conn: &DBConnection,
    ) -> Result<Option<AccountModel>, ErrorInService> {
        let model = rss_account::Entity::find()
            .filter(rss_account::Column::Id.eq(req.id))
            .into_model()
            .one(conn)
            .await
            .map_err(ErrorInService::DBError)?;

        Ok(model)
    }
}
