# Hexo Circle of Friends Simple

这是一个简化版的 Hexo 朋友圈工具，专注于获取友链 RSS 数据并生成 rss.json 文件。

## 功能特点

- 从友链页面爬取友链信息
- 从配置文件中读取自定义友链
- 支持直接指定 RSS 链接
- 过滤屏蔽站点
- 生成标准格式的 rss.json 文件
- 无需数据库，轻量级运行
- 支持获取文章正文内容

## 配置说明

### settings.yaml

主要配置文件，位于 `config/settings.yaml`：

```yaml
# 友链页面配置，使用common1主题
LINK:
  - link: https://example.com/links/  # 友链页面地址

# 友链列表配置
SETTINGS_FRIENDS_LINKS:
  enable: true  # 是否启用自定义友链
  json_api_or_path: ""  # JSON API 地址或本地文件路径
  list:  # 手动配置的友链列表
    - ["博主名称", "博客地址", "头像地址"]
    # 可选的第四项为suffix，自定义RSS订阅后缀，如：
    # - ["博主名称", "博客地址", "头像地址", "feed"]
    # - ["博主名称", "博客地址", "头像地址", "rss.xml"]

# 屏蔽站点列表，支持正则表达式
BLOCK_SITE:
  - example.com

# 每个主页中最多获取几篇文章，请设置一个正整数
MAX_POSTS_NUM: 10

# 清理过期文章天数（简化版仅作配置参考）
OUTDATE_CLEAN: 30
```

### css_rules.yaml

CSS 选择器规则文件，位于 `config/css_rules.yaml`，用于定义如何从页面中提取信息。

## 使用方法

### 1. 安装依赖

```bash
cd simple_edition
cargo build --release
```

### 2. 配置文件

根据你的需求修改 `config/settings.yaml` 文件。

### 3. 运行程序

```bash
cargo run
```

运行完成后，会在当前目录生成 `rss.json` 文件。

## rss.json 格式说明

生成的 rss.json 文件包含以下结构：

```json
{
  "statistical_data": {
    "friends_num": 10,  # 友链总数
    "active_num": 8,    # 活跃友链数
    "error_num": 2,     # 出错友链数
    "article_num": 50,  # 文章总数
    "last_updated_time": "2023-01-01 12:00:00",  # 最后更新时间
    "cache_time": 0
  },
  "article_data": [
    {
      "floor": 1,
      "title": "文章标题",
      "created": "2023-01-01 10:00:00",
      "updated": "2023-01-01 10:00:00",
      "link": "https://example.com/post/1",
      "author": "博主名称",
      "avatar": "https://example.com/avatar.jpg",
      "content": "文章正文内容..."  # 新增字段：文章正文内容
    }
    // 更多文章...
  ]
}
```

## GitHub Action 集成

可以通过 GitHub Action 定期运行程序并将生成的 rss.json 文件部署到指定服务器。详细配置请参考 `.github/workflows/deploy.yml` 示例。