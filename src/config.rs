use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};
use serde_yaml;

/// CSS选择器规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssRules {
    pub post_page_rules: serde_yaml::Value,
    pub link_page_rules: serde_yaml::Value,
}

/// 友链配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendsLinksConfig {
    pub enable: bool,
    pub json_api_or_path: String,
    pub list: Vec<Vec<String>>,
}

/// FC配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcSettings {
    pub enable_link_page: bool,
    pub link_pages: Vec<String>,
    pub settings_friends_links: FriendsLinksConfig,
    pub block_sites: Vec<String>,
    pub max_posts_num: usize,
    pub outdate_clean: usize,
    // 移除simple_mode字段，固化为true
}

/// 从YAML文件读取CSS规则
pub fn get_css_rules(path: &str) -> Result<CssRules, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let rules: serde_yaml::Value = serde_yaml::from_reader(file)?;
    
    let post_page_rules = rules["post_page_rules"].clone();
    let link_page_rules = rules["link_page_rules"].clone();
    
    Ok(CssRules {
        post_page_rules,
        link_page_rules,
    })
}

/// 从YAML文件读取FC配置
pub fn get_fc_settings(path: &str) -> Result<FcSettings, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;
    
    // 友链页配置 - 新格式
    let enable_link_page = true; // 默认启用
    let mut link_pages = Vec::new();
    
    // 从LINK字段读取友链页地址
    if let Some(link_array) = yaml["LINK"].as_vec() {
        for link_item in link_array {
            if let Some(link) = link_item["link"].as_str() {
                link_pages.push(link.to_string());
            }
        }
    }
    
    // 友链列表配置
    let friends_links = &yaml["SETTINGS_FRIENDS_LINKS"];
    let enable = friends_links["enable"].as_bool().unwrap_or(false);
    let json_api_or_path = friends_links["json_api_or_path"].as_str().unwrap_or("")
        .to_string();
    
    let list = friends_links["list"].as_vec().unwrap_or(&vec![])
        .iter()
        .map(|item| {
            item.as_vec().unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .collect();
    
    // 屏蔽站点
    let block_sites = yaml["BLOCK_SITE"].as_vec().unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();
    
    // 其他配置
    let max_posts_num = yaml["MAX_POSTS_NUM"].as_i64().unwrap_or(0) as usize;
    let outdate_clean = yaml["OUTDATE_CLEAN"].as_i64().unwrap_or(0) as usize;
    
    Ok(FcSettings {
        enable_link_page,
        link_pages,
        settings_friends_links: FriendsLinksConfig {
            enable,
            json_api_or_path,
            list,
        },
        block_sites,
        max_posts_num,
        outdate_clean,
        // simple_mode已固化为true
    })
}