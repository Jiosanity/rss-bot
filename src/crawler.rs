use std::collections::HashMap;
use chrono::{FixedOffset, Utc};
use nipper::Document;
use reqwest::Client;
use url::Url;
use crate::models::{Friends, PostMeta};

const BEIJING_OFFSET: Option<FixedOffset> = FixedOffset::east_opt(8 * 3600);

/// 爬取友链页面，获取作者、链接、头像信息
pub async fn crawl_link_page(
    link_page: &str,
    css_rules: &serde_yaml::Value,
    client: &Client,
) -> Result<Vec<Friends>, Box<dyn std::error::Error>> {
    let response = client.get(link_page).send().await?;
    let html = response.text().await?;
    
    let doc = Document::from(&html);
    let mut friends_list = Vec::new();
    
    // 尝试不同主题的CSS选择器规则
    if let Some(theme_rules) = css_rules.get("link_page_rules") {
        for (theme, rules) in theme_rules.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
            if let Some(author_selector) = rules["author"].as_str() {
                    let authors = doc.select(author_selector);
                
                if authors.length() > 0 {
                    // 找到匹配的主题规则
                    let link_selector = rules["link"].as_str().unwrap_or(".friend-name a");
                    let avatar_selector = rules["avatar"].as_str().unwrap_or(".avatar img");
                    
                    let links = doc.select(link_selector);
                    let avatars = doc.select(avatar_selector);
                    
                    let now = Utc::now().with_timezone(&BEIJING_OFFSET.unwrap());
                    let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    
                    // 合并数据
                    let mut author_map = HashMap::new();
                    authors.each(|i, author| {
                        author_map.insert(i, decode_html_entities(author.text().trim()));
                    });
                    
                    let mut link_map = HashMap::new();
                    links.each(|i, link| {
                        if let Some(href) = link.attr("href") {
                            if let Ok(full_url) = resolve_relative_url(href, link_page) {
                                link_map.insert(i, full_url);
                            }
                        }
                    });
                    
                    let mut avatar_map = HashMap::new();
                    avatars.each(|i, avatar| {
                        if let Some(src) = avatar.attr("src") {
                            if let Ok(full_url) = resolve_relative_url(src, link_page) {
                                avatar_map.insert(i, full_url);
                            }
                        }
                    });
                    
                    // 创建Friends对象
                    let max_index = std::cmp::max(
                        std::cmp::max(author_map.len(), link_map.len()),
                        avatar_map.len()
                    );
                    
                    for i in 0..max_index {
                        let name = author_map.get(&i).unwrap_or(&String::from("Unknown")).clone();
                        let link = link_map.get(&i).unwrap_or(&String::from(link_page)).clone();
                        let avatar = avatar_map.get(&i).unwrap_or(&String::from("")).clone();
                        
                        friends_list.push(Friends {
                            name,
                            link,
                            avatar,
                            error: false,
                            created_at: created_at.clone(),
                        });
                    }
                    
                    break; // 找到匹配的规则后停止尝试
                }
            }
        }
    }
    
    Ok(friends_list)
}

/// 爬取文章页面或RSS，获取文章列表
pub async fn crawl_post_page(
    link: &str,
    fc_settings: &crate::config::FcSettings,
    custom_rss: &str,
    css_rules: &serde_yaml::Value,
    client: &Client,
) -> Result<Vec<PostMeta>, Box<dyn std::error::Error>> {
    // 如果提供了自定义RSS，则直接爬取RSS
    if !custom_rss.is_empty() {
        return crawl_post_page_feed(custom_rss, client).await;
    }
    
    // 否则尝试作为RSS链接爬取
    if link.ends_with(".xml") || link.ends_with(".rss") || link.contains("feed") {
        return crawl_post_page_feed(link, client).await;
    }
    
    // 最后尝试爬取HTML页面
    let response = client.get(link).send().await?;
    let html = response.text().await?;
    
    let doc = Document::from(&html);
    let mut posts = Vec::new();
    
    // 尝试不同主题的CSS选择器规则
    if let Some(theme_rules) = css_rules.get("post_page_rules") {
        for (theme, rules) in theme_rules.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
            if let Some(title_selector) = rules["title"].as_str() {
                    let titles = doc.select(title_selector);
                
                if titles.length() > 0 {
                    // 找到匹配的主题规则
                    let link_selector = rules["link"].as_str().unwrap_or(".article-title a");
                    let created_selector = rules["created"].as_str().unwrap_or(".article-date");
                    
                    let links = doc.select(link_selector);
                    let created_times = doc.select(created_selector);
                    
                    // 合并数据
                    let mut title_map = HashMap::new();
                    titles.each(|i, title| {
                        title_map.insert(i, decode_html_entities(title.text().trim()));
                    });
                    
                    let mut link_map = HashMap::new();
                    links.each(|i, link_elem| {
                        if let Some(href) = link_elem.attr("href") {
                            if let Ok(full_url) = resolve_relative_url(href, link) {
                                link_map.insert(i, full_url);
                            }
                        }
                    });
                    
                    let mut time_map = HashMap::new();
                    created_times.each(|i, time_elem| {
                        let time_str = clean_time_string(time_elem.text().trim());
                        time_map.insert(i, time_str);
                    });
                    
                    // 创建PostMeta对象
                    let max_index = std::cmp::max(
                        std::cmp::max(title_map.len(), link_map.len()),
                        time_map.len()
                    );
                    
                    for i in 0..max_index {
                        let title = title_map.get(&i).unwrap_or(&String::from("Untitled")).clone();
                        let post_link = link_map.get(&i).unwrap_or(&String::from(link)).clone();
                        let created = time_map.get(&i).unwrap_or(&String::from("")).clone();
                        
                        // 对于HTML页面，暂时使用空字符串作为内容
                        // 在实际应用中，可以进一步爬取每个文章链接获取详细内容
                        let content = String::new();
                        
                        posts.push(PostMeta {
                            title,
                            link: post_link,
                            created: created.clone(),
                            updated: created,
                            content, // 添加文章正文内容
                        });
                    }
                    
                    break; // 找到匹配的规则后停止尝试
                }
            }
        }
    }
    
    // 限制文章数量
    if fc_settings.max_posts_num > 0 && posts.len() > fc_settings.max_posts_num {
        posts.truncate(fc_settings.max_posts_num);
    }
    
    Ok(posts)
}

/// 爬取RSS订阅源
pub async fn crawl_post_page_feed(
    feed_url: &str,
    client: &Client,
) -> Result<Vec<PostMeta>, Box<dyn std::error::Error>> {
    let response = client.get(feed_url).send().await?;
    let xml = response.text().await?;
    
    let doc = Document::from(&xml);
    let mut posts = Vec::new();
    
    // 尝试RSS格式
    let items = doc.select("item");
    
    items.each(|_, item| {
        let title = item.select("title").text().trim().to_string();
        let link = item.select("link").text().trim().to_string();
        
        // 尝试不同的时间格式
        let mut created = String::new();
        for time_tag in ["pubDate", "date", "dc:date"] {
            if let Some(time_elem) = item.select(time_tag).first() {
                created = parse_rss_time(time_elem.text().trim());
                if !created.is_empty() {
                    break;
                }
            }
        }
        
        // 如果无法解析时间，使用当前时间
        if created.is_empty() {
            let now = Utc::now().with_timezone(&BEIJING_OFFSET.unwrap());
            created = now.format("%Y-%m-%d %H:%M:%S").to_string();
        }
        
        // 提取文章正文内容
        let mut content = String::new();
        // 尝试多种可能的内容标签
        for content_tag in ["content:encoded", "description"] {
            if let Some(content_elem) = item.select(content_tag).first() {
                content = decode_html_entities(content_elem.text().trim());
                if !content.is_empty() {
                    break;
                }
            }
        }
        
        if let Ok(resolved_link) = resolve_relative_url(&link, feed_url) {
            posts.push(PostMeta {
                title: decode_html_entities(&title),
                link: resolved_link,
                created: created.clone(),
                updated: created,
                content, // 添加文章正文内容
            });
        }
    });
    
    Ok(posts)
}

/// 解析RSS时间格式
fn parse_rss_time(time_str: &str) -> String {
    // 尝试多种时间格式
    let formats = [
        "%a, %d %b %Y %H:%M:%S %z",
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S.%3f%z",
        "%Y-%m-%d %H:%M:%S",
    ];
    
    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(time_str, format) {
            return dt.format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }
    
    // 如果都失败了，返回空字符串
    String::new()
}

/// 清理时间字符串
fn clean_time_string(time_str: &str) -> String {
    // 移除多余的空格和特殊字符
    let cleaned = time_str.replace(&[' ', '\t', '\n', '\r', ':', '：'][..], "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string();
    
    // 如果看起来像日期格式，尝试格式化
    if cleaned.len() >= 8 {
        let mut formatted = String::new();
        // 尝试提取年月日
        if cleaned.len() >= 10 {
            // YYYY-MM-DD 或类似格式
            formatted.push_str(&cleaned[0..4]);
            formatted.push('-');
            formatted.push_str(&cleaned[5..7]);
            formatted.push('-');
            formatted.push_str(&cleaned[8..10]);
            formatted.push_str(" 00:00:00");
        } else {
            // 其他格式，直接返回
            return cleaned;
        }
        return formatted;
    }
    
    // 使用当前时间
    let now = Utc::now().with_timezone(&BEIJING_OFFSET.unwrap());
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 解析相对URL为绝对URL
fn resolve_relative_url(relative: &str, base: &str) -> Result<String, Box<dyn std::error::Error>> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }
    
    let base_url = Url::parse(base)?;
    let joined = base_url.join(relative)?;
    Ok(joined.to_string())
}

/// 解码HTML实体
fn decode_html_entities(text: &str) -> String {
    // 简单的HTML实体解码
    let mut result = text.to_string();
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&amp;", "&");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result
}