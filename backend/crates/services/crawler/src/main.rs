use crawler::remove_links::remove_expired_links;
use crawler::utils::{load_categories_from_dir, load_subscriptions_from_dir};

use lib_core::error::ErrorInService;
use lib_core::rss::schema::{
    InsertSubscriptionRecordRequestBuilder, QueryPreferUpdateSubscriptionRequest,
    SubscriptionWithLinksResp,
};
use lib_core::rss::{
    CreateOrUpdateSubscriptionRequest, LinkController, SubscriptionBuildRecordStatus,
};
use lib_core::rss::{SubscriptionController, SubscritionConfigController};
use lib_utils::Setting;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{self, timeout};

use clap::Parser;
use rand::Rng;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    pub cfg_file: Option<String>,
}

pub fn app() -> Cli {
    Cli::parse()
}

pub struct Runner;

impl Runner {
    pub fn get_setting(&self, app: Cli) -> Setting {
        match Setting::from_config(app.cfg_file) {
            Ok(setting) => setting,
            Err(e) => {
                println!("配置文件解析失败:{}， 将使用默认配置运行", e);
                Setting::default()
            }
        }
    }

    // 更新链接的元信息
    pub async fn update_links_meta_job(
        &self,
        links: Vec<lib_core::rss::schema::LinkModel>,
        setting: &Setting,
    ) -> anyhow::Result<()> {
        let js_server_host = match &setting.services.js_server_host {
            Some(js_server_host) => js_server_host,
            None => {
                println!("summary_rss_link error:链接解析服务未就绪");
                return Err(ErrorInService::Custom("链接解析服务未就绪".to_string()).into());
            }
        };
        println!("链接解析服务:{}", js_server_host);
        let ping_url = format!("{}/health/check", js_server_host);
        let ping_result = reqwest::get(ping_url).await;
        if ping_result.is_err() {
            return Err(ErrorInService::Custom("链接解析服务未就就绪".to_string()).into());
        }

        let conn_origin = lib_core::get_db_conn(setting.database.uri.clone()).await;
        let conn_origin_arc = Arc::new(tokio::sync::Mutex::new(conn_origin));

        let links = links
            .iter()
            .map(|link| link.to_owned())
            .collect::<Vec<lib_core::rss::schema::LinkModel>>();

        let (link_update_tx, mut link_update_rx) = tokio::sync::mpsc::channel::<
            Result<lib_core::rss::CreateOrUpdateRssLinkRequest, ErrorInService>,
        >(2);

        let conn_clone = Arc::clone(&conn_origin_arc);
        let handler = tokio::spawn(async move {
            let conn_temp = conn_clone.lock().await;
            while let Some(recv) = link_update_rx.recv().await {
                let result = recv;
                match result {
                    Ok(req) => {
                        let _ = LinkController.insert_link(req, &conn_temp).await;
                    }
                    Err(e) => {
                        tracing::error!("更新链接失败:{}", e);
                    }
                }
            }
        });

        // define tasks for every subscription, which will send data to the channel, send subscription'title to the channel
        let mut tasks = Vec::new();

        let num_cpus = num_cpus::get();
        let max_concurrent_tasks = num_cpus + 2;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks));

        for link in links {
            let request_url = format!("{}/parse", js_server_host);
            let tx = link_update_tx.clone();
            let semaphore = Arc::clone(&semaphore);
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                let result = match crawler::utils::fetch_link_meta(link.link.clone(), request_url)
                    .await
                {
                    Ok(meta) => {
                        let mut update_link_req_builder =
                            lib_core::rss::schema::CreateOrUpdateRssLinkRequestBuilder::default();
                        update_link_req_builder.link(link.link);
                        update_link_req_builder.title(link.title);
                        if link.content.is_none() {
                            if let Some(content) =
                                meta["content"].as_str().map(|content| content.to_string())
                            {
                                update_link_req_builder.description(content.clone());

                                if let Ok(desc_pure_txt) =
                                    lib_crawler::try_get_all_text_from_html_content(content)
                                {
                                    update_link_req_builder.desc_pure_txt(desc_pure_txt);
                                }
                            }
                        }
                        if let Some(lead_image_url) = meta["lead_image_url"]
                            .as_str()
                            .map(|lead_image_url| lead_image_url.to_string())
                        {
                            let image = lib_core::rss::schema::Image {
                                url: lead_image_url.clone(),
                                title: None,
                                link: None,
                                width: None,
                                height: None,
                                description: None,
                            };
                            update_link_req_builder.images(vec![image]);
                        }
                        // 如果没有 Description， 那么尝试使用 content

                        update_link_req_builder.build()
                    }
                    Err(e) => Err(e),
                };
                _ = tx.send(result).await;
            });
            tasks.push(task);
        }

        // wait for all tasks to finish
        for task in tasks {
            task.await?;
        }
        handler.abort_handle().abort();
        Ok(())
    }

    pub async fn update_subscriptions_job(
        &self,
        setting: &Setting,
    ) -> anyhow::Result<Vec<lib_core::rss::schema::LinkModel>> {
        let conn_origin = lib_core::get_db_conn(setting.database.uri.clone()).await;

        // 查询需要更新的订阅源

        let req = QueryPreferUpdateSubscriptionRequest::new(3u32);
        let subs_config_controller = SubscritionConfigController;
        let subscriptions = match subs_config_controller
            .query_prefer_update_subscription(req, &conn_origin)
            .await
        {
            Ok(subscriptions) => subscriptions,
            Err(e) => {
                anyhow::bail!("查询需要更新的订阅源失败:{}", e);
            }
        };

        let update_subscription_ids = subscriptions
            .iter()
            .flat_map(|subscription| subscription.id)
            .collect::<Vec<i64>>();

        // define a safe variable to count the number of updated subscriptions and links
        let inserted_links = Arc::new(tokio::sync::Mutex::new(
            Vec::<lib_entity::rss_link::Model>::new(),
        ));
        let conn_origin_arc = Arc::new(tokio::sync::Mutex::new(conn_origin));
        let updated_subscription_count = Arc::new(Mutex::new(0));

        let update_subscription_failed_count = Arc::new(Mutex::new(0));
        // define a mpsc channel
        let (sub_update_tx, mut sub_update_rx) = tokio::sync::mpsc::channel::<(
            CreateOrUpdateSubscriptionRequest,
            Result<SubscriptionWithLinksResp, ErrorInService>,
        )>(2);

        // define a task to receive data from the channel
        let updated_subscription_count_clone = Arc::clone(&updated_subscription_count);
        let updated_subscription_failed_count_clone = Arc::clone(&update_subscription_failed_count);

        let inserted_links_clone = Arc::clone(&inserted_links);
        let conn_clone = Arc::clone(&conn_origin_arc);
        let handler = tokio::spawn(async move {
            let mut inserted_links_tmp = inserted_links_clone.lock().await;
            let conn_temp = conn_clone.lock().await;
            while let Some(recv) = sub_update_rx.recv().await {
                let (subscription, sub) = recv;
                // 如果订阅源更新失败，那么将更新状态记录到数据库
                let sub = match sub {
                    Ok(sub) => sub,
                    Err(e) => {
                        println!("更新订阅源失败:{}", e);
                        if let Some(id) = subscription.id {
                            if let Ok(req) = InsertSubscriptionRecordRequestBuilder::default()
                                .subscription_id(id)
                                .status(SubscriptionBuildRecordStatus::Faild)
                                .build()
                            {
                                _ = subs_config_controller
                                    .insert_subscription_update_record(req, &conn_temp)
                                    .await;
                            }
                        }
                        let lock = Arc::clone(&updated_subscription_failed_count_clone);
                        let mut count = lock.lock().unwrap();
                        *count += 1;
                        continue;
                    }
                };
                // 更新订阅源
                let SubscriptionWithLinksResp {
                    subscription: rss_subscription,
                    links,
                } = sub;
                let language = match subscription.language {
                    Some(language) => Some(language),
                    None => rss_subscription.language.clone(),
                };
                // 首先更新订阅源部分，更新订阅源的最后更新时间
                let mut update_subscription_req = rss_subscription;
                update_subscription_req.last_build_date = Some(chrono::Utc::now().naive_utc());
                update_subscription_req.category_id = subscription.category_id;
                update_subscription_req.language = language;
                match SubscriptionController
                    .insert_subscription(update_subscription_req, &conn_temp)
                    .await
                {
                    Ok((_, _)) => {
                        let lock = Arc::clone(&updated_subscription_count_clone);
                        let mut count = lock.lock().unwrap();
                        *count += 1;
                    }
                    Err(e) => {
                        let lock = Arc::clone(&updated_subscription_failed_count_clone);
                        let mut count = lock.lock().unwrap();
                        *count += 1;
                        tracing::error!("更新订阅源失败:{}", e);
                        continue;
                    }
                }
                // 尝试更新拉取到的每一条连接，这里的更新是指如果链接不存在，那么将链接插入到数据库中， 如果已经存在了，也不会有后续的操作
                let all_link_count = links.len();

                for link in links {
                    let mut update_link_req = link;
                    update_link_req.subscrption_id = subscription.id;
                    match LinkController
                        .insert_link(update_link_req, &conn_temp)
                        .await
                    {
                        Ok((is_update, l)) => {
                            if !is_update {
                                inserted_links_tmp.push(l);
                            }
                        }
                        Err(e) => {
                            tracing::error!("更新链接失败:{}", e);
                            continue;
                        }
                    }
                }
                // 如果新链接数量大于总链接数量的一半，那么认为更新订阅源成功
                if inserted_links_tmp.len() > all_link_count / 3 {
                    // 更新订阅源成功
                    if let Some(id) = subscription.id {
                        if let Ok(req) = InsertSubscriptionRecordRequestBuilder::default()
                            .subscription_id(id)
                            .status(SubscriptionBuildRecordStatus::Success)
                            .build()
                        {
                            _ = subs_config_controller
                                .insert_subscription_update_record(req, &conn_temp)
                                .await;
                        }
                    }
                }
            }
        });

        // define tasks for every subscription, which will send data to the channel, send subscription'title to the channel
        let mut tasks = Vec::new();

        let num_cpus = num_cpus::get();
        let max_concurrent_tasks = num_cpus + 2;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks));

        for subscription in subscriptions {
            let tx = sub_update_tx.clone();
            let semaphore = Arc::clone(&semaphore);
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                let result = SubscriptionController
                    .parser_rss_from_url(subscription.clone().link.as_str())
                    .await;
                _ = tx.send((subscription.clone(), result)).await;
            });
            tasks.push(task);
        }

        // wait for all tasks to finish
        for task in tasks {
            task.await?;
        }
        handler.abort_handle().abort();
        let inserted_link_ref = inserted_links.lock().await;
        println!(
            "更新订阅源成功:{} 更新订阅源失败:{} 更新链接成功:{}",
            updated_subscription_count.lock().unwrap(),
            update_subscription_failed_count.lock().unwrap(),
            inserted_link_ref.len()
        );
        let conn_outside = conn_origin_arc.lock().await;
        if let Err(err) = SubscritionConfigController
            .update_subscription_config(update_subscription_ids.clone(), &conn_outside)
            .await
        {
            println!("更新订阅源失败:{}", err);
        }
        // 返回所有新增的链接模型
        let links = inserted_link_ref
            .iter()
            .map(|link| link.clone().into())
            .collect();
        Ok(links)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runner = Runner {};
    let mut app = Cli::parse();
    app.cfg_file = Some("fixture/config.toml".to_string());

    let setting = match Setting::from_config(app.clone().cfg_file) {
        Ok(setting) => setting,
        Err(e) => {
            println!("配置文件解析失败:{}， 将使用默认配置运行", e);
            tracing::error!("配置文件解析失败:{}， 将使用默认配置运行", e);
            Setting::default()
        }
    };
    Setting::set_global(setting.clone());
    let conn = lib_core::get_db_conn(setting.database.uri.clone()).await;

    let workspace = std::path::Path::new("data");
    if !workspace.exists() {
        std::fs::create_dir(workspace).unwrap();
    }
    println!("开始加载分类");

    let category_dir = "fixture/rss/categories";
    let categories = match load_categories_from_dir(category_dir.to_string(), &conn).await {
        Ok(categories) => categories,
        Err(e) => {
            tracing::error!("加载分类失败:{}", e);
            vec![]
        }
    };

    tracing::info!("开始加载订阅源");
    let subscription_dir = "fixture/rss/feeds";
    match load_subscriptions_from_dir(subscription_dir.to_string(), categories, &conn).await {
        Ok(_) => {
            tracing::info!("加载订阅源成功");
        }
        Err(e) => {
            println!("加载订阅源失败:{}", e);
            tracing::error!("加载订阅源失败:{}", e);
        }
    }

    let setting = runner.get_setting(app.clone());
    // 开启一个定时任务，每隔一段时间更新一次订阅源
    #[allow(clippy::manual_map)]
    loop {
        let min_sec = 60 * 10; // 最小秒数为1小时

        let max_sec = 60 * 40; // 最大秒数为3小时

        let sec = rand::thread_rng().gen_range(min_sec..=max_sec);

        let before_jobs = chrono::Utc::now();
        println!("现在是:{} 准备开始更新订阅源任务", before_jobs);

        let six_month_ago = before_jobs - chrono::Duration::days(180);
        _ = remove_expired_links(workspace, &six_month_ago.naive_utc(), &conn).await;

        let links = runner.update_subscriptions_job(&setting).await?;
        let afte_update_subscriptions_at = chrono::Utc::now();
        let link_spent = afte_update_subscriptions_at - before_jobs;
        let link_spent_sec = link_spent.num_seconds();
        let local_date = chrono::Local::now();
        println!(
            "更新订阅源任务完成, 本次更新耗时:{}秒, 现在是:{}, 将在{}秒后再次更新订阅源",
            link_spent_sec, local_date, sec
        );
        let interval = Duration::from_secs(sec); // 每隔一小时更新一次

        // 过滤掉
        let mut filted_links = links
            .iter()
            .filter(|link| {
                link.images
                    .as_ref()
                    .map_or(true, |images| images.is_empty())
            })
            .cloned()
            .collect::<Vec<lib_core::rss::schema::LinkModel>>();
        // 根据 `published_at` 排序
        filted_links.sort_by(|a, b| {
            // compare published_at
            let a = a
                .published_at
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());
            let b = b
                .published_at
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());

            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        // 只根据 1s 一个链接的速度计算，空余时间可以接受的更新数量
        let trimmed_links: Vec<lib_core::rss::schema::LinkModel> = filted_links;
        println!(
            "准备更新链接信息任务, 本次更新链接数量:{}",
            trimmed_links.len()
        );
        // 更新链接信息，这里限定一个时间，如果超过这个时间，那么将会终止任务
        _ = timeout(
            interval.saturating_sub(Duration::from_secs(60)),
            runner.update_links_meta_job(trimmed_links, &setting),
        )
        .await;

        let after_update_links_at = chrono::Utc::now();
        let update_links_spent_sec =
            after_update_links_at.timestamp() - afte_update_subscriptions_at.timestamp();

        // 这里睡眠的时间需要减少更新链接所消耗的时间了
        let mut real_interval = interval.as_secs() as i64 - update_links_spent_sec;
        if real_interval < 10 {
            real_interval = 10;
        }
        println!(
            "更新链接信息任务完成, 本次更新耗时:{}秒, 现在是:{}, 将休眠:{}秒后再次更新",
            update_links_spent_sec,
            chrono::Local::now(),
            real_interval
        );

        time::sleep(Duration::from_secs(real_interval as u64)).await;
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        // let url = "https://www.wmagazine.com/rss";
        // let url = "https://feeds.bbci.co.uk/sport/football/rss.xml";
        let url = "https://cnnespanol.cnn.com/feed/";

        let channel = SubscriptionController.parser_rss_from_url(url).await;
        let channel = match channel {
            Ok(channel) => channel,
            Err(e) => {
                println!("解析订阅源失败:{}", e);
                return;
            }
        };
        let SubscriptionWithLinksResp {
            subscription: rss_subscription,
            links,
        } = channel;
        println!("subscription.title :{:?}", rss_subscription.title);
        println!(
            "subscription.description :{:?}",
            rss_subscription.description
        );
        println!("subscription.link :{:?}", rss_subscription.link);
        println!(
            "subscription.last_build_date :{:?}",
            rss_subscription.last_build_date
        );
        println!("links.len :{:?}", links.len());
    }

    #[tokio::test]
    async fn test_run_counter() {
        let counter = Arc::new(Mutex::new(0));
        // count to 100 with 2 tasks
        let counter1 = Arc::clone(&counter);
        let task1 = tokio::spawn(async move {
            for _ in 0..50 {
                let mut count = counter1.lock().unwrap();
                *count += 1;
            }
        });
        let counter2 = Arc::clone(&counter);
        let task2 = tokio::spawn(async move {
            for _ in 0..50 {
                let mut count = counter2.lock().unwrap();
                *count += 1;
            }
        });
        task1.await.unwrap();
        task2.await.unwrap();
        let count = counter.lock().unwrap();
        assert_eq!(*count, 100);
    }
}
