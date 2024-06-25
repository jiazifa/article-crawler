use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SummaryLinkRequest {
    // 必须是url，不能和用户绑定，否则用户切换设备后，缓存失效
    pub link_url: String,
    // 总结内容 (如果不为空，则不去解析缓存或者地址了， 说明这是一个客户端解析过的文章正文)
    pub link_content: Option<String>,
}
