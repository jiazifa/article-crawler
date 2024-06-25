use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use lib_utils::Setting;

use crate::{api_error, response::APIResponse};

/// 验证请求头中的签名
/// 用于验证这个请求是否是从指定的客户端发出的
async fn validate_request_header_sign(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let headers = request.headers();
    let required_header = vec![
        "OS".to_string().to_uppercase(),
        "TS".to_string().to_uppercase(),
        "LANG".to_string().to_uppercase(),
        "VN".to_string().to_uppercase(),
        "VC".to_string().to_uppercase(),
    ];

    for header in required_header.iter() {
        if !headers.contains_key(header) {
            let resp = APIResponse::<()>::new()
                .with_error(api_error::APIError::ErrorParams(header.clone()));
            return Err(resp.into_response());
        }
    }

    let setting = Setting::global();
    let secret = setting.jwt.secret.clone();
    // GET Values
    let mut origin_string = String::new();

    if let Some(os_value) = headers.get("OS") {
        origin_string.push_str(os_value.to_str().unwrap_or_default());
    }
    if let Some(ts_value) = headers.get("TS") {
        origin_string.push_str(ts_value.to_str().unwrap_or_default());
    }
    if let Some(lang_value) = headers.get("LANG") {
        origin_string.push_str(lang_value.to_str().unwrap_or_default());
    }
    if let Some(vn_value) = headers.get("VN") {
        origin_string.push_str(vn_value.to_str().unwrap_or_default());
    }
    origin_string.push_str(&secret);
    // 组装完成
    let vc_value = match headers.get("VC") {
        Some(vc_value) => vc_value.to_str().unwrap_or_default(),
        None => "",
    };

    let digest = md5::compute(origin_string.as_bytes());
    let md5_value = format!("{:x}", digest);
    if !md5_value.eq(vc_value) {
        let resp = APIResponse::<()>::new()
            .with_error(api_error::APIError::ErrorParams("校验失败".to_string()));
        return Err(resp.into_response());
    }

    Ok(next.run(request).await)
}
