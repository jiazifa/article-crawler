use crate::schema::Category;
use lib_core::{
    common_schema::PageRequest,
    error::ErrorInService,
    feed::{
        schema::{
            CreateOrUpdateCategoryRequestBuilder, CreateOrUpdateSubscriptionRequestBuilder,
            QueryCategoryRequestBuilder,
        },
        CategoryController, SubscriptionController,
    },
    DBConnection,
};
use serde_json::json;

pub async fn load_categories_from_dir(
    category_dir: String,
    conn: &DBConnection,
) -> Result<Vec<Category>, anyhow::Error> {
    // 列出所有的文件
    let categories_files: Vec<String> = match std::fs::read_dir(category_dir) {
        Ok(files) => files
            .filter_map(|file| {
                let file = file.ok()?;
                let path = file.path();
                if path.extension()?.to_str()? == "json" {
                    path.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>(),
        Err(e) => {
            println!("读取分类文件失败:{}", e);
            tracing::error!("读取分类文件失败:{}", e);
            vec![]
        }
    };

    let mut categories: Vec<Vec<Category>> = Vec::new();
    for f in categories_files {
        let category_json = match load_categories(f, &conn).await {
            Ok(category_json) => category_json,
            Err(e) => {
                println!("读取json文件失败:{}", e);
                tracing::error!("读取json文件失败:{}", e);
                continue;
            }
        };
        categories.push(category_json);
    }
    let flatten_categories = categories
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<Category>>();

    return Ok(flatten_categories);
}

pub async fn load_categories(
    category_file: String,
    conn: &DBConnection,
) -> Result<Vec<Category>, anyhow::Error> {
    let category_controller = CategoryController;
    let category_json = match std::fs::read_to_string(category_file.clone()) {
        Ok(category_json) => match serde_json::from_str::<Category>(category_json.as_str()) {
            Ok(category_json) => vec![category_json],
            Err(e) => {
                println!("解析json 文件失败{} / {}", category_file, e);
                tracing::error!("解析json文件失败:{}", e);
                vec![]
            }
        },
        Err(e) => {
            tracing::error!("读取json文件失败{}:{}", category_file, e);
            vec![]
        }
    };
    for category in category_json.clone() {
        let mut root_category_req_builder = CreateOrUpdateCategoryRequestBuilder::default();
        root_category_req_builder.title(category.name.clone());
        if let Some(value) = category.description {
            root_category_req_builder.description(value);
        }
        if let Some(value) = category.sort_order {
            root_category_req_builder.sort_order(value);
        }
        let root_category_req = match root_category_req_builder.build() {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("构建分类失败:{}", e);
                continue;
            }
        };
        let root_category = match category_controller
            .insert_category(root_category_req, conn)
            .await
        {
            Ok(root_category) => root_category,
            Err(e) => {
                tracing::error!("创建分类失败:{}", e);
                continue;
            }
        };
        if let Some(children) = category.children {
            for child in children {
                let description = match child.description {
                    Some(description) => match description.as_str() {
                        "" => Some(format!("{} -{}", category.name, child.name)),
                        _ => Some(description),
                    },
                    None => Some(format!("{} -{}", category.name, child.name)),
                };

                let mut child_category_req_builder =
                    CreateOrUpdateCategoryRequestBuilder::default();
                child_category_req_builder.title(child.name.clone());

                if let Some(value) = description {
                    child_category_req_builder.description(value);
                }
                if let Some(value) = child.sort_order {
                    child_category_req_builder.sort_order(value);
                }
                child_category_req_builder.parent_id(root_category.id);
                let child_category_req = child_category_req_builder.build().unwrap();
                let _ = category_controller
                    .insert_category(child_category_req, conn)
                    .await
                    .unwrap();
            }
        }
    }

    let all_categories = category_json;
    let category_flattened = all_categories
        .iter()
        .flat_map(|category| {
            let mut categories = vec![category.clone()];
            if let Some(children) = category.children.clone() {
                categories.extend(children);
            }
            categories
        })
        .collect::<Vec<Category>>();
    Ok(category_flattened)
}

pub async fn load_subscriptions_from_dir(
    subscription_dir: String,
    categories: Vec<Category>,
    conn: &DBConnection,
) -> Result<(), anyhow::Error> {
    let subscriptions_files: Vec<String> = match std::fs::read_dir(subscription_dir) {
        Ok(files) => files
            .filter_map(|file| {
                let file = file.ok()?;
                let path = file.path();
                if path.extension()?.to_str()? == "json" {
                    path.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>(),
        Err(e) => {
            tracing::error!("读取订阅源文件失败:{}", e);
            vec![]
        }
    };

    for f in subscriptions_files {
        match load_subscriptions(f, categories.clone(), conn).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("读取订阅源失败:{}", e);
                continue;
            }
        }
    }
    Ok(())
}

pub async fn load_subscriptions(
    json_file: String,
    categories: Vec<Category>,
    conn: &DBConnection,
) -> Result<(), anyhow::Error> {
    let category_flattened = categories;
    let json = match std::fs::read_to_string(json_file) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("读取json文件失败:{}", e);
            std::process::exit(1);
        }
    };
    // load category json

    // 输出 json 的key
    // may array or object
    let json_value: serde_json::Value = match serde_json::from_str(json.as_str()) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("解析json文件失败:{}", e);
            std::process::exit(1);
        }
    };

    let get_category_by_id = |category: &str| -> Option<Category> {
        let category_int = category.parse::<i32>().unwrap();

        // find category from category_json, or its children

        let target_category = category_flattened
            .iter()
            .find(|category| category.id == category_int);
        target_category.cloned()
    };

    let category_controller = CategoryController;
    let query_category_req = QueryCategoryRequestBuilder::default()
        .page(PageRequest::max_page())
        .build()
        .unwrap();
    let all_categories_in_db = match category_controller
        .query_category(query_category_req, conn)
        .await
    {
        Ok(all_categories_in_db) => all_categories_in_db.to_owned(),
        Err(e) => {
            tracing::error!("查询分类失败:{}", e);
            std::process::exit(1);
        }
    };

    // JSON 是一个字典，key是字符，value是另一个字典
    let mut insert_item_count = 0u32;

    if let Some(items) = json_value.as_array() {
        for item in items {
            let category_id = item.get("categoryId").unwrap().as_i64().unwrap();
            let self_category_in_json = get_category_by_id(category_id.to_string().as_str());
            let self_category = match self_category_in_json {
                Some(c) => all_categories_in_db
                    .iter()
                    .find(|c_db| c_db.title == c.name),
                None => None,
            };

            let title = match item.get("title") {
                Some(title) => title.as_str().unwrap_or_default(),
                None => "",
            };

            // feedUrl
            let feed_link = match item.get("feedUrl") {
                Some(link) => link.as_str().unwrap_or_default(),
                None => "",
            };

            let icon_url = item
                .get("iconUrl")
                .map(|i| i.as_str().unwrap_or_default().to_string());

            // 优先使用 logo，其次使用 iconUrl
            let logo = match item.get("logo") {
                Some(logo) => Some(logo.as_str().unwrap_or_default().to_string()),
                None => icon_url,
            };

            let language = item
                .get("language")
                .map(|language| language.as_str().unwrap_or_default().to_string());

            let description = item
                .get("description")
                .map(|description| description.as_str().unwrap_or_default().to_string());

            let website: Option<String> = item
                .get("website")
                .map(|website| website.as_str().unwrap_or_default().to_string());

            let visual_url: Option<String> = item
                .get("visualUrl")
                .map(|x| x.as_str().unwrap_or_default().to_string());

            let mut sub_req_builder = CreateOrUpdateSubscriptionRequestBuilder::default();
            sub_req_builder.title(title.to_string());

            if let Some(value) = description {
                sub_req_builder.description(value);
            }
            sub_req_builder.link(feed_link);

            if let Some(value) = website {
                sub_req_builder.site_link(value);
            }

            if let Some(c) = self_category {
                sub_req_builder.category_id(c.id);
            }

            if let Some(value) = logo {
                sub_req_builder.logo(value);
            }

            if let Some(value) = visual_url {
                sub_req_builder.visual_url(value);
            }

            if let Some(value) = language {
                sub_req_builder.language(value);
            }
            let sub_req = match sub_req_builder.build() {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("构建订阅源失败:{}", e);
                    continue;
                }
            };

            let _ = SubscriptionController
                .insert_subscription(sub_req, conn)
                .await
                .unwrap();
            insert_item_count += 1;
        }

        tracing::info!("insert {} items", insert_item_count);
    }
    Ok(())
}

pub async fn fetch_link_meta(
    url: String,
    request_url: String,
) -> Result<serde_json::Value, ErrorInService> {
    let resp = reqwest::Client::new()
        .post(request_url)
        .json(&json!({ "url": url, "ignore_cache": true}))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("fetch_link_meta error:{}", e);
            ErrorInService::Custom("请求解析链接失败".to_string())
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            tracing::error!("fetch_link_meta error:{}", e);
            ErrorInService::Custom("解析链接失败".to_string())
        })?;

    Ok(resp)
}
