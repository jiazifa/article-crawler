pub use rss::{extension::ExtensionBuilder, extension::ExtensionMap, Channel};
use std::collections::BTreeMap;
use std::path::Path;

use crate::get_content_from_url;
use crate::url::RequestOptionBuilder;

pub async fn fetch_rss_from_url<T: AsRef<str>>(url: T) -> anyhow::Result<Channel> {
    let path = format!("{}.xml", url.as_ref().replace('/', "_").replace(':', ""));
    let content = match Path::new(&path).exists() {
        true => std::fs::read_to_string(&path)?,
        false => {
            // 重试3次
            let req = RequestOptionBuilder::default()
                .url(url.as_ref().to_string())
                .timeout(15)
                .retry_times(3)
                .build()
                .map_err(|e| anyhow::anyhow!(e))?;
            get_content_from_url(req).await?
            // 根据url 写到当前目录的文件中
            // 如果本地已经存在，就不写了
            // if !Path::new(&path).exists() {
            //     // 写到文件中
            //     _ = std::fs::write(path, content.clone());
            // }
        }
    };

    // 如果content 是空的，就返回错误
    if content.is_empty() {
        return Err(anyhow::anyhow!("rss content is empty"));
    }

    // 首先定义一系列的尝试解析的策略，每一个策略都是一个函数，返回一个Option<Channel>，如果解析成功，就返回Some(Channel)，否则返回None
    // 依次尝试每一个策略，如果有一个策略成功，就返回，否则返回错误
    // 这个判断的规则是 channel.validate() 返回的结果，如果是Err，就说明解析失败，如果是Ok，就说明解析成功

    let try_ops = vec![
        // 尝试通过 feed_rs 解析
        |content: String| -> anyhow::Result<Channel> {
            return parse_rss_by_feed_rs(content.as_bytes()).map_err(|e| anyhow::anyhow!(e));
        },
        // 尝试通过 rss 解析
        |content: String| -> anyhow::Result<Channel> {
            return Channel::read_from(content.as_bytes()).map_err(|e| anyhow::anyhow!(e));
        },
    ];

    // channel is first no none value of try_ops
    let mut error = None;
    let mut channel = None;
    for op in try_ops {
        match op(content.clone()) {
            Ok(c) => {
                channel = Some(c);
                break;
            }
            Err(e) => {
                error = Some(e);
                continue;
            }
        }
    }

    match channel {
        Some(c) => {
            // 如果解析成功，就返回
            Ok(c)
        }
        None => match error {
            Some(e) => {
                // 如果解析失败，就返回错误
                Err(e)
            }
            None => {
                // 如果没有解析失败，也没有解析成功，就返回错误
                Err(anyhow::anyhow!("rss 解析失败"))
            }
        },
    }
}

fn parse_rss_by_feed_rs<R: std::io::Read>(content: R) -> anyhow::Result<Channel> {
    let feed = feed_rs::parser::parse(content).map_err(|e| anyhow::anyhow!(e))?;
    let channel = map_feed_model_to_channel(feed)?;
    Ok(channel)
}

fn map_feed_model_to_channel(feed: feed_rs::model::Feed) -> anyhow::Result<Channel> {
    let title = match feed.title {
        Some(title) => title.content,
        None => {
            return Err(anyhow::anyhow!("rss title 不存在"));
        }
    };
    let description = match feed.description {
        Some(description) => description.content,
        None => "".to_string(),
    };
    let mut channel = Channel::default();
    channel.set_title(title);
    channel.set_description(description);
    channel.set_link(match feed.links.first() {
        Some(link) => link.href.clone(),
        None => "".to_string(),
    });
    channel.set_language(feed.language);
    channel.set_last_build_date(match feed.updated {
        Some(updated) => updated.to_rfc3339(),
        None => "".to_string(),
    });
    channel.set_pub_date(match feed.published {
        Some(published) => published.to_rfc3339(),
        None => "".to_string(),
    });
    let feed_items: Vec<rss::Item> = feed
        .entries
        .into_iter()
        .map(|element| {
            let mut item = rss::Item::default();

            // 设置标题
            item.set_title(match element.title {
                Some(title) => Some(title.content),
                None => Some("".to_string()),
            });

            // 设置描述
            item.set_description(match element.summary {
                Some(summary) => Some(summary.content),
                None => Some("".to_string()),
            });

            // 设置链接
            item.set_link(match element.links.first() {
                Some(link) => Some(link.href.clone()),
                None => Some("".to_string()),
            });

            // 设置作者
            item.set_author(match element.authors.first() {
                Some(author) => Some(author.name.clone()),
                None => Some("".to_string()),
            });

            // 设置发布日期
            let pub_date = match element.published {
                Some(published) => Some(published),
                None => element.updated,
            };

            item.set_pub_date(pub_date.map(|pub_date| pub_date.to_rfc3339()));

            let mut extension_map = BTreeMap::new();
            // 设置图片, 从 media 中获取
            let mut images = Vec::new();
            for media in element.media {
                if let Some(first) = media.thumbnails.first() {
                    // get first url
                    let ext = ExtensionBuilder::default()
                        .value(Some(first.image.uri.clone()))
                        .build();
                    images.push(ext);
                    // 设置 enclosure
                    let mut enclosure = rss::Enclosure::default();
                    enclosure.set_url(first.image.uri.clone());
                    item.set_enclosure(enclosure);
                }
            }
            extension_map.insert("images".to_string(), images);
            // 设置作者的详细信息
            let author_detail = element
                .authors
                .iter()
                .map(|author| {
                    let mut author_meta = BTreeMap::new();
                    author_meta.insert("name".to_string(), author.name.clone());

                    if let Some(uri) = author.uri.clone() {
                        author_meta.insert("uri".to_string(), uri);
                    }

                    if let Some(email) = author.email.clone() {
                        author_meta.insert("email".to_string(), email);
                    }
                    ExtensionBuilder::default().attrs(author_meta).build()
                })
                .collect::<Vec<_>>();
            extension_map.insert("authors".to_string(), author_detail);
            // category
            extension_map.insert(
                "category".to_string(),
                element
                    .categories
                    .iter()
                    .map(|c| {
                        let mut category_meta = BTreeMap::new();
                        category_meta.insert("term".to_string(), c.term.clone());
                        category_meta
                            .insert("scheme".to_string(), c.scheme.clone().unwrap_or_default());
                        category_meta
                            .insert("label".to_string(), c.label.clone().unwrap_or_default());
                        ExtensionBuilder::default().attrs(category_meta).build()
                    })
                    .collect::<Vec<_>>(),
            );

            let mut ext = ExtensionMap::default();
            ext.insert("ext".to_string(), extension_map);

            item.set_extensions(ext);
            item
        })
        .collect();
    channel.set_items(feed_items);
    Ok(channel)
}

// test
#[tokio::test]
async fn test_fetch_rss_from_url() {
    let url = "https://rss.uol.com.br/feed/noticias.xml";
    let channel = fetch_rss_from_url(url).await;
    assert!(channel.is_ok());
    let channel_value = channel.unwrap();

    println!("channel_value:title :{:?}", channel_value.title());
    println!(
        "channel_value:description :{:?}",
        channel_value.description()
    );
    println!("channel_value:link :{:?}", channel_value.link());
    println!(
        "channel_value:last_build_date :{:?}",
        channel_value.last_build_date()
    );
    println!("channel_value:pub_date :{:?}", channel_value.pub_date());
    println!("channel_value:language :{:?}", channel_value.language());
    println!("channel_value:items :{:?}", channel_value.items().len());
    assert!(!channel_value.items().is_empty());
}
