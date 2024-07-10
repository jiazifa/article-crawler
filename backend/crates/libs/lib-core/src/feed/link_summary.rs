use super::schema::{
    CreateAiTokenRecordRequest, LinkMindMapRequest, LinkSummaryModel, LinkSummaryRequest,
};
use crate::error::ErrorInService;
use crate::DBConnection;
use lib_entity::feed_link_summary;
use lib_openai::{AISummaryController, Config};
use sea_orm::{entity::*, query::*};
use serde::Deserialize;

// 更新文章总结模型
#[derive(Debug, Deserialize)]
struct UpdateLinkSummaryRequest {
    pub link_url: String,
    // 总结语言
    pub language: Option<String>,
    // 总结提供者 (例如 gpt-3.5-turbo)
    pub provider: Option<String>,
    // 一句话总结
    pub one_sentence_summary: String,
    // 关键点
    pub key_points: Option<String>,
    // 行动项
    pub action_items: Option<String>,
    // 关键词
    pub keywords: Option<String>,
}

/// 更新文章总结模型
#[derive(Debug, Deserialize)]
struct UpdateLinkMindMapRequest {
    pub link_url: String,
    // 总结语言
    pub language: Option<String>,
    // 总结要点的json字符
    pub mind_map: String,
}

pub struct LinkSummaryController;

/// Inserts a new link summary into the database or updates an existing one.
///
/// # Arguments
///
/// * `req` - The `UpdateLinkSummaryRequest` containing the information for the link summary.
/// * `conn` - The database connection.
///
/// # Returns
///
/// Returns a `Result` containing the inserted or updated `LinkSummaryModel`, or an `ErrorInService` if there was an error.
///
/// # Example
///
/// ```rust
/// let controller = LinkSummaryController::new();
/// let request = UpdateLinkSummaryRequest {
///     link_url: "https://example.com".to_string(),
///     one_sentence_summary: "This is a summary.".to_string(),
///     key_points: vec!["Point 1".to_string(), "Point 2".to_string()],
///     action_items: vec!["Action 1".to_string(), "Action 2".to_string()],
///     keywords: vec!["keyword1".to_string(), "keyword2".to_string()],
///     language: Some("English".to_string()),
///     provider: Some("Example Provider".to_string()),
/// };
/// let conn = get_db_connection();
/// let result = controller.insert_link_summary(request, &conn);
/// match result {
///     Ok(summary) => println!("Link summary inserted/updated: {:?}", summary),
///     Err(error) => eprintln!("Error inserting/updating link summary: {:?}", error),
/// }
/// ```
impl LinkSummaryController {
    async fn insert_link_summary_db(
        &self,
        req: UpdateLinkSummaryRequest,
        conn: &DBConnection,
    ) -> Result<LinkSummaryModel, ErrorInService> {
        let query = feed_link_summary::Entity::find()
            .filter(feed_link_summary::Column::LinkUrl.eq(req.link_url.clone()));

        let summary = query.one(conn).await.map_err(ErrorInService::DBError)?;
        let should_update = summary.is_some();
        let mut new_model = match summary {
            Some(m) => m.into_active_model(),
            None => feed_link_summary::ActiveModel {
                link_url: Set(req.link_url.clone()),
                version: Set(Some("1".to_string())),
                ..Default::default()
            },
        };
        new_model.summary = Set(Some(req.one_sentence_summary.clone()));
        new_model.key_points = Set(req.key_points.clone());
        new_model.action_items = Set(req.action_items.clone());
        new_model.keywords = Set(req.keywords.clone());
        if let Some(ref l) = req.language {
            new_model.language = Set(Some(l.clone()));
        }
        if let Some(provider) = &req.provider {
            new_model.provider = Set(Some(provider.clone()));
        }

        let updated = match should_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };
        Ok(updated.into())
    }
}

impl LinkSummaryController {
    pub async fn find_summary_by_link_url(
        &self,
        link_url: String,
        conn: &DBConnection,
    ) -> Result<Option<feed_link_summary::Model>, ErrorInService> {
        let query = feed_link_summary::Entity::find()
            .filter(feed_link_summary::Column::LinkUrl.eq(link_url));

        let summary = query.one(conn).await.map_err(ErrorInService::DBError)?;
        Ok(summary)
    }

    pub async fn insert_link_summary<C>(
        &self,
        req: LinkSummaryRequest,
        config: C,
        conn: &DBConnection,
    ) -> Result<LinkSummaryModel, ErrorInService>
    where
        C: Config,
    {
        // 首先尝试查找摘要
        let summary = self
            .find_summary_by_link_url(req.link_url.clone(), conn)
            .await?;
        // 存在缓存的总结，直接返回
        if let Some(s) = summary {
            return Ok(s.into());
        }
        let content = match req.content {
            None => {
                return Err(ErrorInService::Custom(
                    "link summary content is empty".to_string(),
                ))
            }
            Some(ref c) => c.clone(),
        };

        let controller = AISummaryController::new(config);

        let summary = controller.generate_summary(content).await.map_err(|e| {
            ErrorInService::Custom(format!("openai generate article summary failed:{}", e))
        })?;

        let req = UpdateLinkSummaryRequest {
            link_url: req.link_url.clone(),
            provider: summary.provider,
            language: summary.language,
            one_sentence_summary: summary.summary,
            key_points: serde_json::to_string(&summary.key_points).ok(),
            action_items: serde_json::to_string(&summary.action_items).ok(),
            keywords: serde_json::to_string(&summary.keywords).ok(),
        };
        let summary = self.insert_link_summary_db(req, conn).await?;
        Ok(summary)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_link_summary() {
        let raw_json_str = "{\n    \"one_sentence_summary\": \"The article discusses the transformative impact of the Software Revolution and the key drivers of innovation, emphasizing the great future in software development.\",\n    \"key_points\": [\n        \"The Software Revolution is defined as the transformational force of a world dominated by software, driven by the democratization of software and a massive global appetite for new applications.\",\n        \"Key drivers of innovation in software development include human creativity, access to resources, diversity, and success models.\",\n        \"Product teams struggle to build the right applications due to soft strategies, weak tools, and squishy communications.\",\n        \"Product managers and their teams have been neglected in terms of business applications, but there is now a demand for tools to help them create brilliant roadmaps and enhance their productivity.\",\n        \"Aha! is a solution specifically built for product and engineering managers to set clear strategies, define releases, and detail key user stories in a dynamic software development environment.\"\n    ],\n    \"action_items\": [\n        \"Recognize the transformative impact of the Software Revolution and its significance in driving innovation and progress.\",\n        \"Explore the key drivers of innovation in software development, including human creativity, access to resources, diversity, and success models, to understand the foundation of the industry.\",\n        \"Recognize the challenges faced by product teams and understand the demand for specialized tools to enhance their productivity and efficiency.\",\n        \"Consider implementing specialized software, such as Aha!, to assist product and engineering managers in setting clear strategies and defining key user stories in a dynamic software development environment.\",\n        \"Reflect on the great future in software and its impact on society and businesses, considering the opportunities it presents for growth and success.\"\n    ],\n    \"keywords\": [\n        \"Software Revolution\",\n        \"innovation\",\n        \"product management\",\n        \"Aha!\",\n        \"software development\"\n    ]\n}";
        let json = serde_json::from_str::<serde_json::Value>(raw_json_str).unwrap();
        println!("json: {:?}", json);
        let one_sentence_summary_raw = match json["one_sentence_summary"].as_str() {
            Some(s) => s.to_string(),
            None => "".to_string(),
        };
        let key_points_raw = match json["key_points"].as_array() {
            Some(s) => serde_json::to_string(s).unwrap_or("".to_string()),
            None => "".to_string(),
        };
        let action_items_raw = match json["action_items"].as_array() {
            Some(s) => serde_json::to_string(s).unwrap_or("".to_string()),
            None => "".to_string(),
        };
        let keywords_raw = match json["keywords"].as_array() {
            Some(s) => serde_json::to_string(s).unwrap_or("".to_string()),
            None => "".to_string(),
        };
        // println!("one_sentence_summary_raw: {:?}", one_sentence_summary_raw);
        // println!("key_points_raw: {:?}", key_points_raw);
        // println!("action_items_raw: {:?}", action_items_raw);
        // println!("keywords_raw: {:?}", keywords_raw);
        assert!(!one_sentence_summary_raw.is_empty());
        assert!(!key_points_raw.is_empty());
        assert!(!action_items_raw.is_empty());
        assert!(!keywords_raw.is_empty());
    }
}
