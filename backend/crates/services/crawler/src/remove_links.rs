use std::{fmt::Debug, path::Path};

use chrono::NaiveDateTime;
use lib_core::{feed::LinkController, DBConnection};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveExpiredLinkRecord {
    pub(crate) removed_at: NaiveDateTime,
    pub(crate) removed_count: Option<i64>,
}

impl RemoveExpiredLinkRecord {
    pub fn new(removed_at: NaiveDateTime, removed_count: Option<i64>) -> Self {
        Self {
            removed_at,
            removed_count,
        }
    }
}

// 加载已经移除的链接记录
async fn load_removed_links_records(
    workspace: &Path,
) -> anyhow::Result<Vec<RemoveExpiredLinkRecord>> {
    let path = workspace.join("removed_links_records.json");
    let content = tokio::fs::read_to_string(path).await?;
    let records: Vec<RemoveExpiredLinkRecord> = serde_json::from_str(&content)?;
    Ok(records)
}

// 保存已经移除的链接记录
async fn save_removed_links_records(
    workspace: &Path,
    records: Vec<RemoveExpiredLinkRecord>,
) -> anyhow::Result<()> {
    let path = workspace.join("removed_links_records.json");
    let content = serde_json::to_string_pretty(&records)?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

// 追加已经移除的链接记录
async fn append_removed_links_records(
    workspace: &Path,
    record: RemoveExpiredLinkRecord,
) -> anyhow::Result<()> {
    let mut records = match load_removed_links_records(workspace).await {
        Ok(records) => records,
        Err(_) => vec![],
    };
    records.push(record);
    save_removed_links_records(workspace, records).await?;
    Ok(())
}

pub async fn remove_expired_links(
    workspace: &Path,
    expired_at: &NaiveDateTime,
    conn: &DBConnection,
) -> anyhow::Result<()> {
    let now = chrono::Local::now().naive_local();
    let records = match load_removed_links_records(workspace).await {
        Ok(records) => records,
        Err(_) => vec![],
    };

    // 如果本月已经移除过链接，则不再移除
    if records.iter().any(|record| {
        format!("{}", record.removed_at.format("%Y-%m")) == format!("{}", now.format("%Y-%m"))
    }) {
        return Ok(());
    }

    let removed_count = LinkController
        .remove_expired_links(*expired_at, conn)
        .await?;

    let now = chrono::Local::now().naive_local();
    // 保存已经移除的链接记录
    let record = RemoveExpiredLinkRecord::new(now, Some(removed_count as i64));
    append_removed_links_records(workspace, record).await?;
    Ok(())
}
