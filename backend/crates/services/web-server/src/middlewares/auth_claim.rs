use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use hyper::{header, HeaderMap};
use jsonwebtoken as jwt;
use jwt::Algorithm;
use jwt::Validation;
use lib_core::{
    auth::{
        schema::{CustomerModel, QueryCustomerByIDRequest},
        CustomerController,
    },
    DBConnection,
};
use lib_entity::rss_customer;
use lib_utils::Setting;
use serde::{Deserialize, Serialize};

use crate::api_error;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    // "Summer"
    pub iss: Option<String>,
    // "auth"
    pub sub: Option<String>,
    // users
    pub aud: Option<Vec<String>>,
    // 1600000000
    pub exp: usize,
    // 1600000000
    pub nbf: usize,
    // 1600000000
    pub iat: usize,
    // "token_auth"
    pub jti: String,
}
pub fn ecode_to_jwt(auth: &AuthClaims, secret: &[u8]) -> Option<String> {
    match jwt::encode(
        &jwt::Header::default(),
        &auth,
        &jwt::EncodingKey::from_secret(secret),
    ) {
        Ok(token) => Some(token),
        Err(e) => {
            tracing::error!("jwt encode error:{:?}", e);
            None
        }
    }
}

pub fn decode_to_claims(token: &str, secret: &[u8]) -> Result<AuthClaims, api_error::APIError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["users"]);

    match jwt::decode::<AuthClaims>(token, &jwt::DecodingKey::from_secret(secret), &validation) {
        Ok(data) => Ok(data.claims),
        Err(e) => {
            tracing::error!("jwt decode error:{:?}", e);
            Err(api_error::APIError::ErrorParams(
                "jwt decode error".to_string(),
            ))
        }
    }
}

impl AuthClaims {
    pub fn new(
        iss: Option<String>,
        sub: Option<String>,
        aud: Option<Vec<String>>,
        exp: usize,
        nbf: usize,
        iat: usize,
        jti: String,
    ) -> Self {
        Self {
            iss,
            sub,
            aud,
            exp,
            nbf,
            iat,
            jti,
        }
    }

    pub fn encode(&self, secret: &[u8]) -> Option<String> {
        super::auth_claim::ecode_to_jwt(self, secret)
    }

    /// Extract claims from request headers
    pub fn extract_from_request(
        headers: &HeaderMap,
        decoding_key: String,
    ) -> Result<Self, api_error::APIError> {
        let claim = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                let words = h.split("Bearer").collect::<Vec<&str>>();
                words.get(1).map(|w| w.trim())
            })
            .map(|token| decode_to_claims(token, decoding_key.as_bytes()));
        match claim {
            Some(claim) => claim,
            None => Err(api_error::APIError::ErrorParams("claims".to_string())),
        }
    }

    pub async fn get_user(
        &self,
        pool: &DBConnection,
    ) -> Result<CustomerModel, api_error::APIError> {
        let uid = self
            .jti
            .parse::<i64>()
            .map_err(|_| api_error::APIError::ErrorParams("id".to_string()))?;
        let controller = CustomerController;
        let req = QueryCustomerByIDRequest::new(uid);

        let op_u = controller.query_customer_by_id(req, pool).await?;
        match op_u {
            Some(u) => Ok(u),
            None => Err(api_error::APIError::Toast("用户不存在".to_string())),
        }
    }
}

#[async_trait]
impl<B> FromRequestParts<B> for AuthClaims
where
    B: Send + Sync,
{
    type Rejection = api_error::APIError;

    async fn from_request_parts(parts: &mut Parts, state: &B) -> Result<Self, Self::Rejection> {
        // You can either call them directly...
        if !parts.headers.contains_key("Token") {
            return Err(api_error::APIError::ErrorParams("Token".to_string()));
        }
        let token = parts
            .headers
            .get("Token")
            .ok_or(api_error::APIError::ErrorParams("Token".to_string()))?
            .to_str()
            .unwrap_or("");
        // let TypedHeader(Authorization(bearer)) =
        //     TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
        //         .await
        //         .map_err(|_e| api_error::APIError::ErrorParams("Authorization".to_string()))?;

        // let token = bearer.token();
        let setting = Setting::global();

        let secret = setting.jwt.secret.clone();
        let info = match decode_to_claims(token, secret.as_bytes()) {
            Ok(info) => info,
            Err(e) => return Err(e),
        };
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    #[test]
    fn test_auth_claims_new() {
        let issuer = "Summer".to_string();
        let subject = "auth".to_string();
        let audience = vec!["users".to_string()];
        let expires_at = (Utc::now().timestamp() + 3600) as usize; // 1 hour from now
        let not_before = Utc::now().timestamp() as usize;
        let issued_at = Utc::now().timestamp() as usize;
        let id = "123".to_string();

        let auth_claims = AuthClaims::new(
            Some(issuer.clone()),
            Some(subject.clone()),
            Some(audience.clone()),
            expires_at,
            not_before,
            issued_at,
            id.clone(),
        );

        assert_eq!(auth_claims.iss, Some(issuer.clone()));
        assert_eq!(auth_claims.sub, Some(subject.clone()));
        assert_eq!(auth_claims.aud, Some(audience.clone()));
        assert_eq!(auth_claims.jti, id);

        let secret = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>();
        let token = auth_claims.encode(secret.as_bytes());
        assert!(token.is_some());
        let token = token.unwrap();
        let decoded_claims = decode_to_claims(&token, secret.as_bytes()).unwrap();
        println!("decoded_claims: {:?}", decoded_claims);
        assert_eq!(decoded_claims.iss, Some(issuer));
    }
}
