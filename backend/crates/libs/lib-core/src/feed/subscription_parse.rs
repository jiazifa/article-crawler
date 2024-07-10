use super::schema::{
    self, Author, CreateOrUpdateRssLinkRequest, CreateOrUpdateRssLinkRequestBuilder,
    CreateOrUpdateSubscriptionRequest, CreateOrUpdateSubscriptionRequestBuilder, Image,
    QueryPreferUpdateSubscriptionRequest, QueryRssLinkRequestBuilder, QuerySubscriptionRequest,
    QuerySubscriptionRequestBuilder, QuerySubscriptionsWithLinksRequest, SubscriptionModel,
    SubscriptionWithLinksResp, UpdateSubscriptionCountRequest,
};
use crate::error::ErrorInService;
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike};
use lib_crawler::{try_get_all_image_from_html_content, try_get_all_text_from_html_content};
use std::collections::{BTreeMap, HashSet};

pub struct SubscriptionParseController;

impl SubscriptionParseController {
    /// 从 url 中解析 rss 订阅源
    pub async fn parser_rss_from_url<T: AsRef<str>>(
        url: T,
    ) -> Result<SubscriptionWithLinksResp, ErrorInService> {
        let rss_feed = lib_crawler::fetch_rss_from_url(url.as_ref())
            .await
            .map_err(|e| ErrorInService::Custom(format!("解析RSS失败:{}", e)))?;
        let mut links: Vec<CreateOrUpdateRssLinkRequest> = Vec::new();
        let pub_date = match rss_feed.pub_date() {
            Some(d) => match dateparser::parse(d) {
                Ok(d) => Some(d.to_utc().naive_utc()),
                Err(_) => None,
            },
            None => None,
        };
        let language = rss_feed.language().map(|l| l.to_string());

        // 构建订阅源
        let mut subscription_req = CreateOrUpdateSubscriptionRequestBuilder::default();
        subscription_req.title(rss_feed.title().to_string());

        subscription_req.description(rss_feed.description().to_string());
        subscription_req.link(url.as_ref().to_string());

        subscription_req.site_link(rss_feed.link().to_string());
        if let Some(value) = pub_date {
            subscription_req.pub_date(value);
        }
        if let Some(value) = language {
            subscription_req.language(value);
        }
        let subscription = subscription_req.clone().build()?;

        let current_timestamp = chrono::Utc::now().timestamp();

        // 遍历所有的item
        rss_feed.items().iter().for_each(|item| {
            // 没有链接的item，直接返回
            let item_link = match item.link() {
                Some(link) => link,
                None => return,
            };
            // 出版时间，如果没有默认使用当前时间
            let pub_date = match item.pub_date() {
                Some(d) => match dateparser::parse(d) {
                    Ok(d) => Some(d.naive_utc()), // 包装在Some中
                    Err(_) => DateTime::from_timestamp(current_timestamp, 0)
                        .and_then(|d| Some(d.to_utc().naive_utc())),
                    // 修改这里，确保返回Option
                },
                None => DateTime::from_timestamp(current_timestamp, 0)
                    .and_then(|d| Some(d.to_utc().naive_utc())),
            };
            // 解析额外参数
            let ext = item.extensions();
            let mut ext_map: BTreeMap<String, Vec<rss::extension::Extension>> = BTreeMap::new();
            if let Some(e_map) = ext.get("ext") {
                ext_map = e_map.to_owned();
            }
            let authors_json = match ext_map.get("authors") {
                Some(authors) => {
                    let mut authors_j = Vec::new();
                    // insert from extension's attr
                    authors.iter().for_each(|author| {
                        let attr = author.attrs();
                        authors_j.push(attr);
                    });
                    serde_json::to_string(&authors_j).ok()
                }
                None => None,
            };
            let mut images_json = match ext_map.get("images") {
                Some(images) => {
                    let mut urls = Vec::new();
                    images.iter().for_each(|image| {
                        if let Some(url) = image.value() {
                            urls.push(url.to_string());
                        }
                    });
                    serde_json::to_string(&urls).ok()
                }
                None => None,
            };
            // 如果没有图片，尝试从描述中解析
            if let (true, Some(desc)) = (images_json.is_none(), item.description()) {
                if let Ok(images) = try_get_all_image_from_html_content(desc.to_string()) {
                    let images = images
                        .iter()
                        .map(|i| rss::Image {
                            url: i.to_string(),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();
                    images_json = serde_json::to_string(&images).ok();
                }
            }

            // 解析纯文本
            let mut pure_desc: Option<String> = None;

            if let Some(desc) = item.description() {
                if let Ok(desc) = try_get_all_text_from_html_content(desc.to_string()) {
                    pure_desc = Some(desc);
                }
            }

            // build feed_item links
            let mut link_req = CreateOrUpdateRssLinkRequestBuilder::default();

            link_req.title(match item.title() {
                Some(t) => t.to_string(),
                None => "".to_string(),
            });
            link_req.link(item_link.to_string());
            if let Some(value) = item.description.clone() {
                link_req.description(value);
            }

            if let Some(value) = pure_desc {
                link_req.desc_pure_txt(value);
            }

            if let Some(value) = images_json {
                let images: Vec<Image> =
                    serde_json::from_str::<Vec<Image>>(&value).unwrap_or_default();
                link_req.images(images.iter().map(|i| i.clone()).collect::<Vec<_>>());
            }
            if let Some(value) = authors_json {
                let authors = serde_json::from_str::<Vec<Author>>(&value).unwrap_or_default();
                link_req.authors(authors.iter().map(|a| a.clone()).collect::<Vec<_>>());
            }
            if let Some(value) = pub_date {
                link_req.published_at(value);
            }
            if let Ok(link) = link_req
                .build()
                .map_err(|e| ErrorInService::Custom(format!("构建链接失败:{}", e)))
            {
                links.push(link);
            }
        });
        let resp = SubscriptionWithLinksResp {
            subscription,
            links,
        };
        Ok(resp)
    }
}
