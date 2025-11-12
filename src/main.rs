// 导出需要的类型和函数
mod models;
mod config;
mod crawler;
mod downloader;

use std::fs::File;
use chrono::{Utc, FixedOffset};
use tokio::task;
use crate::config::{get_css_rules, get_fc_settings};
use crate::downloader::{build_client, start_crawl_linkpages, start_crawl_postpages, start_get_friends_links_from_json};
use crate::models::{AllPostData, Posts};

const BEIJING_OFFSET: Option<FixedOffset> = FixedOffset::east_opt(8 * 3600);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    let now = Utc::now().with_timezone(&BEIJING_OFFSET.unwrap());
    tracing::info!("Starting hexo-circle-of-friends-simple at {}", now.format("%Y-%m-%d %H:%M:%S"));
    
    // 读取配置文件
    tracing::info!("Loading configuration files");
    
    // 获取当前工作目录，构建配置文件的绝对路径
    // 这样可以适应不同的环境，如GitHub Action
    let current_dir = std::env::current_dir()?;
    let css_rules_path = current_dir.join("config").join("css_rules.yaml");
    let settings_path = current_dir.join("config").join("settings.yaml");
    
    tracing::info!("CSS rules path: {}", css_rules_path.display());
    tracing::info!("Settings path: {}", settings_path.display());
    
    let css_rules = get_css_rules(css_rules_path.to_str().ok_or("Failed to convert path to string")?)?;
    let fc_settings = get_fc_settings(settings_path.to_str().ok_or("Failed to convert path to string")?)?;
    
    // 构建HTTP客户端
    let client = build_client(10, 3);
    
    // 爬取友链页面
    let format_base_friends = start_crawl_linkpages(&fc_settings, &css_rules, &client).await;
    tracing::info!("Crawled {} friends from link pages", format_base_friends.len());
    
    // 处理配置项友链
    let mut all_friends = format_base_friends;
    if fc_settings.settings_friends_links.enable {
        tracing::info!("Processing configured friends links");
        let mut settings_friends = Vec::new();
        
        // 处理JSON API或文件中的友链
        if !fc_settings.settings_friends_links.json_api_or_path.is_empty() {
            match start_get_friends_links_from_json(
                &fc_settings.settings_friends_links.json_api_or_path,
                &client
            ).await {
                Ok(json) => {
                    if let Some(friends) = json.get("friends").and_then(|f| f.as_array()) {
                        for friend in friends {
                            if let (Some(name), Some(link)) = (
                                friend.get("name").and_then(|n| n.as_str()),
                                friend.get("link").and_then(|l| l.as_str())
                            ) {
                                let avatar = friend.get("avatar").and_then(|a| a.as_str()).unwrap_or("");
                                settings_friends.push(crate::models::Friends {
                                    name: name.to_string(),
                                    link: link.to_string(),
                                    avatar: avatar.to_string(),
                                    error: false,
                                    created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                                });
                            }
                        }
                        tracing::info!("Loaded {} friends from JSON API/file", settings_friends.len());
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load friends from JSON API/file: {}", e);
                }
            }
        }
        
        // 处理手动配置的友链
        for postpage_vec in &fc_settings.settings_friends_links.list {
            if postpage_vec.len() >= 3 {
                settings_friends.push(crate::models::Friends {
                    name: postpage_vec[0].clone(),
                    link: postpage_vec[1].clone(),
                    avatar: postpage_vec[2].clone(),
                    error: false,
                    created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
        
        // 合并友链并去重
        all_friends.extend(settings_friends);
        all_friends.sort_by(|a, b| a.link.cmp(&b.link));
        all_friends.dedup_by(|a, b| a.link == b.link);
        tracing::info!("Total friends after merging: {}", all_friends.len());
    }
    
    // 爬取文章
    tracing::info!("Starting to crawl articles");
    let mut tasks = Vec::new();
    
    for friend in &all_friends {
        let fc_settings_clone = fc_settings.clone();
        let client_clone = client.clone();
        let css_rules_clone = css_rules.clone();
        let friend_clone = friend.clone();
        
        let task = task::spawn(async move {
            let result = start_crawl_postpages(
                &friend_clone.link,
                &fc_settings_clone,
                String::new(), // 不使用自定义RSS
                &css_rules_clone,
                &client_clone,
            ).await;
            
            // 将错误转换为字符串以满足Send trait要求
            let result_str = match result {
                Ok(posts) => Ok(posts),
                Err(e) => Err(format!("{:?}", e)),
            };
            
            (friend_clone, result_str)
        });
        
        tasks.push(task);
    }
    
    // 收集爬取结果
    let mut success_posts = Vec::new();
    let mut active_num = 0;
    let mut error_num = 0;
    
    for task in tasks {
        match task.await {
            Ok((friend, result_str)) => {
                match result_str {
                    Ok(posts) => {
                        if !posts.is_empty() {
                            active_num += 1;
                            // 转换为Posts对象
                            let posts_with_author: Vec<Posts> = posts
                                .into_iter()
                                .map(|post| Posts {
                                    meta: post,
                                    author: friend.name.clone(),
                                    avatar: friend.avatar.clone(),
                                    created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                                })
                                .collect();
                            let posts_count = posts_with_author.len();
                            success_posts.extend(posts_with_author);
                            tracing::info!("Crawled {} posts from {}", posts_count, friend.name);
                        } else {
                            error_num += 1;
                            tracing::warn!("No posts found for {}", friend.name);
                        }
                    }
                    Err(e) => {
                        error_num += 1;
                        tracing::error!("Failed to crawl posts from {}: {}", friend.name, e);
                    }
                }
            }
            Err(e) => {
                error_num += 1;
                tracing::error!("Task failed: {}", e);
            }
        }
    }
    
    // 按更新时间排序文章
    success_posts.sort_by(|a, b| {
        b.meta.updated.cmp(&a.meta.updated)
    });
    
    // 生成rss.json
    tracing::info!("Generating rss.json");
    let data = AllPostData::new(
        all_friends.len(),
        active_num,
        error_num,
        success_posts.len(),
        now.format("%Y-%m-%d %H:%M:%S").to_string(),
        success_posts,
        0,
    );
    
    // 写入文件
    let file = File::create("rss.json")?;
    serde_json::to_writer_pretty(file, &data)?;
    tracing::info!("Data successfully written to rss.json");
    
    Ok(())
}