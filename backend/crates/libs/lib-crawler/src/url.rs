use std::collections::HashMap;

use derive_builder::Builder;
use rand::seq::SliceRandom;
use rand::Rng;
use scraper::Html;

static FAKE_UAS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36 Edg/112.0.1722.64",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 Edg/119.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.84 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.77 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.102 Safari/537.36 Edge/18.19577",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML like Gecko) Chrome/51.0.2704.79 Safari/537.36 Edge/14.14931",
    "Mozilla/5.0 (iPad; CPU OS 6_0 like Mac OS X) AppleWebKit/536.26 (KHTML, like Gecko) Version/6.0 Mobile/10A5355d Safari/8536.25",
    "Mozilla/5.0 (Windows; U; Windows NT 6.1; tr-TR) AppleWebKit/533.20.25 (KHTML, like Gecko) Version/5.0.4 Safari/533.20.27",
    "Mozilla/5.0 (Windows; U; Windows NT 6.1; zh-HK) AppleWebKit/533.18.1 (KHTML, like Gecko) Version/5.0.2 Safari/533.18.5",
];

#[derive(Debug, Clone, Builder)]
pub struct RequestOption {
    pub url: String,
    #[builder(setter(strip_option), default = "Some(15)")]
    pub timeout: Option<u64>,
    #[builder(setter(strip_option), default)]
    pub retry_times: Option<u8>,
    #[builder(setter(strip_option), default)]
    pub user_agent: Option<String>,
    #[builder(setter(strip_option), default)]
    pub ip_proxy: Option<String>,
    // Referer
    #[builder(setter(strip_option), default = "true")]
    pub referer: bool,
}

pub async fn get_content_from_url(req: RequestOption) -> Result<String, anyhow::Error> {
    let mut retry = req.retry_times.unwrap_or(5);
    let mut content = "".to_string();
    // 默认超时1s，每重试一次，超时时间为 上次的2倍
    let mut delay_secs = 1;
    while retry > 0 {
        match try_get_content_from_url_once(req.clone()).await {
            Ok(body) => {
                content = body;
                break;
            }
            Err(_) => {
                retry -= 1;
                delay_secs *= 2;
                // set max delay time to 30s
                if delay_secs > 20 {
                    // 使用 随机数
                    delay_secs = rand::thread_rng().gen_range(2..10);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            }
        }
    }
    Ok(content)
}

async fn try_get_content_from_url_once(req: RequestOption) -> Result<String, anyhow::Error> {
    let mut client_builder = reqwest::Client::builder();
    if let Some(timeout) = req.timeout {
        client_builder = client_builder.timeout(std::time::Duration::from_secs(timeout));
    }
    if let Some(ip_proxy) = req.ip_proxy {
        client_builder = client_builder.proxy(reqwest::Proxy::all(ip_proxy)?);
    }
    if let Some(user_agent) = req.user_agent {
        client_builder = client_builder.user_agent(user_agent);
    } else {
        // 随机选择一个ua
        let ua = Vec::from_iter(FAKE_UAS)
        .choose(&mut rand::thread_rng())
        .map_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36 Edg/112.0.1722.64", |x| *x).to_string();
        client_builder = client_builder.user_agent(ua);
    }
    // 配置referer
    client_builder = client_builder.referer(req.referer);

    let client = match client_builder.build() {
        Ok(client) => client,
        Err(e) => {
            return Err(anyhow::anyhow!("构建client失败:{}", e));
        }
    };
    let url = req.url;
    let response = client.get(&url).send().await?;
    let body = response.text().await?;
    if body.is_empty() {
        return Err(anyhow::anyhow!("rss内容为空"));
    }
    Ok(body)
}
