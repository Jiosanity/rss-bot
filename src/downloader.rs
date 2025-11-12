use std::time::Duration;
use reqwest::Client;
use crate::config::{FcSettings, CssRules};
use crate::crawler::{crawl_link_page, crawl_post_page};
use crate::models::{Friends, PostMeta};
use tokio::task;

/// 构建HTTP客户端
pub fn build_client(timeout: u64, retry_count: u32) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(timeout))
        .connect_timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .expect("Failed to build HTTP client")
}

/// 开始爬取友链页面
pub async fn start_crawl_linkpages(
    fc_settings: &FcSettings,
    css_rules: &CssRules,
    client: &Client,
) -> Vec<Friends> {
    let mut all_friends = Vec::new();
    
    if fc_settings.enable_link_page {
        for link_page in &fc_settings.link_pages {
            match crawl_link_page(link_page, &serde_yaml::Value::Null, client).await {
                Ok(friends) => {
                    // 过滤掉被屏蔽的站点
                    let filtered_friends: Vec<Friends> = friends
                        .into_iter()
                        .filter(|friend| {
                            !fc_settings.block_sites.iter().any(|block| {
                                friend.link.contains(block)
                            })
                        })
                        .collect();
                    all_friends.extend(filtered_friends);
                }
                Err(e) => {
                    eprintln!("Failed to crawl link page {}: {}", link_page, e);
                }
            }
        }
    }
    
    // 去重
    all_friends.sort_by(|a, b| a.link.cmp(&b.link));
    all_friends.dedup_by(|a, b| a.link == b.link);
    
    all_friends
}

/// 开始爬取文章页面
pub async fn start_crawl_postpages(
    link: &str,
    fc_settings: &FcSettings,
    custom_rss: String,
    css_rules: &CssRules,
    client: &Client,
) -> Result<Vec<PostMeta>, Box<dyn std::error::Error>> {
    // 检查是否在屏蔽列表中
    if fc_settings.block_sites.iter().any(|block| link.contains(block)) {
        return Ok(Vec::new());
    }
    
    crawl_post_page(link, fc_settings, &custom_rss, &serde_yaml::Value::Null, client).await
}

/// 从JSON API或文件获取友链列表
pub async fn start_get_friends_links_from_json(
    json_api_or_path: &str,
    client: &Client,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if json_api_or_path.starts_with("http") {
        // 从API获取
        let response = client.get(json_api_or_path).send().await?;
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        // 从文件读取
        let file_content = std::fs::read_to_string(json_api_or_path)?;
        let json: serde_json::Value = serde_json::from_str(&file_content)?;
        Ok(json)
    }
}