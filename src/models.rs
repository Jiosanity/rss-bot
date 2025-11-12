use serde::{Serialize, Deserialize};

/// 文章元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostMeta {
    pub title: String,
    pub link: String,
    pub created: String,
    pub updated: String,
    pub content: String, // 文章正文内容
}

/// 文章数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Posts {
    pub meta: PostMeta,
    pub author: String,
    pub avatar: String,
    pub created_at: String,
}

/// 友链数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Friends {
    pub name: String,
    pub link: String,
    pub avatar: String,
    pub error: bool,
    pub created_at: String,
}

/// 文章数据，用于JSON输出
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleData {
    pub floor: usize,
    pub title: String,
    pub created: String,
    pub updated: String,
    pub link: String,
    pub author: String,
    pub avatar: String,
    pub content: String, // 文章正文内容
}

impl ArticleData {
    fn new(
        floor: usize,
        title: String,
        created: String,
        updated: String,
        link: String,
        author: String,
        avatar: String,
        content: String,
    ) -> Self {
        ArticleData {
            floor,
            title,
            created,
            updated,
            link,
            author,
            avatar,
            content,
        }
    }
}

/// 统计数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticalData {
    friends_num: usize,
    active_num: usize,
    error_num: usize,
    article_num: usize,
    last_updated_time: String,
}

impl StatisticalData {
    fn new(
        friends_num: usize,
        active_num: usize,
        error_num: usize,
        article_num: usize,
        last_updated_time: String,
    ) -> Self {
        StatisticalData {
            friends_num,
            active_num,
            error_num,
            article_num,
            last_updated_time,
        }
    }
}

/// 所有文章数据，用于JSON输出
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllPostData {
    pub statistical_data: StatisticalData,
    pub article_data: Vec<ArticleData>,
}

impl AllPostData {
    pub fn new(
        friends_num: usize,
        active_num: usize,
        error_num: usize,
        article_num: usize,
        last_updated_time: String,
        posts: Vec<Posts>,
        start_offset: usize,
    ) -> AllPostData {
        let article_data: Vec<ArticleData> = posts
            .into_iter()
            .enumerate()
            .map(|(floor, posts)| {
                ArticleData::new(
                    floor + start_offset + 1,
                    posts.meta.title,
                    posts.meta.created,
                    posts.meta.updated,
                    posts.meta.link,
                    posts.author,
                    posts.avatar,
                    posts.meta.content, // 传递content字段
                )
            })
            .collect();
        AllPostData {
            statistical_data: StatisticalData::new(
                friends_num,
                active_num,
                error_num,
                article_num,
                last_updated_time,
            ),
            article_data,
        }
    }
}