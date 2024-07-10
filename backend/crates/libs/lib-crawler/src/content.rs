use sanitize_html::rules::predefined::RESTRICTED;
use sanitize_html::sanitize_str;
use scraper::Html;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct HtmlMetadata {
    raw: HashMap<String, String>,
}

impl HtmlMetadata {
    pub fn update_if_not_exists(&self, k: String, v: String) -> Self {
        let mut new = self.clone();
        new.raw.entry(k).or_insert(v);
        new
    }

    pub fn update(&self, k: String, v: String) -> Self {
        let mut new = self.clone();
        new.raw.insert(k, v);
        new
    }

    pub fn value(&self, k: &str) -> Option<String> {
        self.raw.get(k).map(|x| x.to_string())
    }

    pub fn title(&self) -> Option<String> {
        self.value("title")
    }

    pub fn description(&self) -> Option<String> {
        self.value("description")
    }

    pub fn image(&self) -> Option<String> {
        self.value("image")
    }

    pub fn url(&self) -> Option<String> {
        self.value("url")
    }

    pub fn icon(&self) -> Option<String> {
        self.value("icon")
    }
}

/// 尝试从url获取metadata
pub async fn try_get_metadata_from_content(content: String) -> Result<HtmlMetadata, anyhow::Error> {
    let html = Html::parse_document(&content);
    let mut meta = HtmlMetadata::default();

    // get title
    let title = html
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(|x| x.inner_html())
        .unwrap_or("".to_string());
    meta = meta.update_if_not_exists("title".to_string(), title);

    let selectors: Vec<(String, Vec<&str>)> = vec![
        (
            "url".to_string(),
            vec![
                "meta[name=url]",
                "meta[name='og:url']",
                "meta[name='twitter:url']",
                "meta[name='weibo:article:url']",
            ],
        ),
        (
            "title".to_string(),
            vec![
                "title",
                "meta[name=title]",
                "meta[name='og:title']",
                "meta[name='twitter:title']",
                "meta[name='weibo:article:title']",
            ],
        ),
        (
            "description".to_string(),
            vec![
                "meta[name=description]",
                "meta[name='og:description']",
                "meta[name='twitter:description']",
                "meta[name='weibo:article:description']",
            ],
        ),
        (
            "icon".to_string(),
            vec![
                "meta[name=icon]",
                "meta[name='og:icon']",
                "meta[name='twitter:icon']",
                "meta[name='weibo:article:icon']",
            ],
        ),
        (
            "image".to_string(),
            vec![
                "meta[name=image]",
                "meta[name=promote_image]",
                "meta[name='og:image']",
                "meta[name='twitter:image']",
                "meta[name='weibo:article:image']",
            ],
        ),
    ];
    // get description
    let description = html
        .select(&scraper::Selector::parse("meta[name=description]").unwrap())
        .next()
        .map(|x| x.value().attr("content").unwrap_or("").to_string());

    for (k, v) in selectors {
        if meta.value(&k).is_some() {
            continue;
        }
        let mut value = "".to_string();
        for selector in v {
            let s = match scraper::Selector::parse(selector) {
                Ok(selector) => selector,
                Err(e) => {
                    println!("parse selector error:{}", e);
                    continue;
                }
            };
            let node = html.select(&s).next();
            if let Some(node) = node {
                value = node.value().attr("content").unwrap_or("").to_string();
                break;
            }
        }
        meta = meta.update_if_not_exists(k, value);
    }
    Ok(meta)
}

pub fn try_get_all_image_from_html_content(content: String) -> Result<Vec<String>, anyhow::Error> {
    let html = Html::parse_document(&content);
    let mut images = Vec::new();
    let selectors: Vec<&str> = vec!["img[src]"];

    // 首先选择 body 的范围
    let body = scraper::Selector::parse("body")
        .map_err(|e| anyhow::anyhow!("parse selector error:{}", e))?;
    // 首先判断是否有 body 标签, 如果有，则优先处理 body 标签内的内容，否则还是处理 html
    let mut body_node = html.select(&body).next();
    if body_node.is_none() {
        body_node = html
            .select(
                &scraper::Selector::parse("html")
                    .map_err(|e| anyhow::anyhow!("parse selector error:{}", e))?,
            )
            .next();
    }

    if let Some(node) = body_node {
        for selector in selectors {
            let s = match scraper::Selector::parse(selector) {
                Ok(selector) => selector,
                Err(e) => continue,
            };
            for node in node.select(&s) {
                if let Some(src) = node.value().attr("src") {
                    if !src.is_empty() {
                        images.push(src.to_string());
                    }
                }
            }
        }
    }

    Ok(images)
}
/// # Examples
///
/// ``` rust
/// let head_content = r#"
/// <ul></ul>
/// <p>Pep Guardiola<a href=\"https://www.theguardian.com/football/2023/dec/22/manchester-city-fluminense-club-world-cup-final-match-report\">City</p>
/// <a href=\"https://www.theguardian.com/football/2023/dec/26/pep-guardiola-doubt-message-manchester-city-critics\"></a>
/// "#;
/// let text = try_get_all_text_from_html_content(head_content.to_string()).unwrap();
/// assert_eq!(text, "\n        Pep GuardiolaCity\n        \n        ");
/// ```
pub fn try_get_all_text_from_html_content(html_content: String) -> Result<String, anyhow::Error> {
    let html = Html::parse_document(&html_content);
    let mut text_content = "".to_string();
    let selectors = vec![
        "div.article-content",
        "div.article",
        "div.content",
        "div.main",
        "div.main-content",
        "div.main-content-inner",
        "div.main-inner",
        "div.main-inner-content",
        "div.main-inner-content-inner",
    ];

    let body = scraper::Selector::parse("body")
        .map_err(|e| anyhow::anyhow!("parse selector error:{}", e))?;
    let body_node = html.select(&body).next().or_else(|| {
        html.select(&scraper::Selector::parse("html").unwrap())
            .next()
    });

    if let Some(node) = body_node {
        for selector in selectors {
            if !text_content.is_empty() {
                break;
            }
            let s = match scraper::Selector::parse(selector) {
                Ok(selector) => selector,
                Err(_) => continue,
            };
            if let Some(node) = node.select(&s).next() {
                let text = node.inner_html();
                let parsed = match sanitize_html::sanitize_str(&RESTRICTED, &text) {
                    Ok(parsed) => parsed,
                    Err(_) => body_node
                        .map(|x| x.text().collect::<Vec<_>>().join("\n"))
                        .unwrap_or("".to_string()),
                };
                text_content = text;
                break;
            }
        }
    }

    if text_content.is_empty() {
        text_content = body_node
            .map(|x| x.text().collect::<Vec<_>>().join(""))
            .unwrap_or("".to_string());
    }

    Ok(text_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_metadata_from_url_content() {
        let head_content = r#"
        <html>
        <head>
        <title>test title</title>
        <meta name="description" content="test description">
        <meta name="og:image" content="test image">
        <meta name="twitter:image" content="test twitter image">
        </head>
        <body>
        </body>
        </html>
        "#;
        let metadata = try_get_metadata_from_content(head_content.to_string())
            .await
            .unwrap();
        assert_eq!(metadata.title().unwrap(), "test title");
        assert_eq!(metadata.description().unwrap(), "test description");
        assert_eq!(metadata.image().unwrap(), "test image");
    }

    fn test_get_images_from_url_content() {
        let head_content = r#"
        <div>
        <img src="test image 1">
        <img src="test image 2">
        <img src="test image 3">
        </div>
        "#;
        let images = try_get_all_image_from_html_content(head_content.to_string()).unwrap();
        assert_eq!(images.len(), 3);
        assert_eq!(images[0], "test image 1");
        assert_eq!(images[1], "test image 2");
        assert_eq!(images[2], "test image 3");
    }

    fn test_get_txt_from_url_content() {
        let head_content = r#"
        <ul></ul>
        <p>Pep Guardiola<a href=\"https://www.theguardian.com/football/2023/dec/22/manchester-city-fluminense-club-world-cup-final-match-report\">City</p>
        <a href=\"https://www.theguardian.com/football/2023/dec/26/pep-guardiola-doubt-message-manchester-city-critics\"></a>
        "#;
        let text = try_get_all_text_from_html_content(head_content.to_string()).unwrap();

        assert_eq!(text, "\n        Pep GuardiolaCity\n        \n        ");
    }
}
