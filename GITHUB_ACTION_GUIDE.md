# GitHub Action 使用指南

本文档详细说明如何配置和使用GitHub Action将生成的`data.json`文件自动传送到指定服务器。

## 1. 配置 GitHub Secrets

在你的GitHub仓库中，需要配置以下Secrets（前往 Settings > Secrets and variables > Actions > New repository secret）：

### 对于 SCP 部署方式（推荐）

- `SERVER_HOST`: 目标服务器的IP地址或域名
- `SERVER_USERNAME`: 服务器登录用户名
- `SSH_PRIVATE_KEY`: 用于SSH登录的私钥（需要在服务器上配置对应的公钥）
- `SERVER_PORT`（可选）: SSH端口，默认22
- `DEPLOY_PATH`（可选）: 目标服务器上保存`data.json`的路径，默认`/var/www/html`

### 对于 API 上传方式

- `API_TOKEN`: 访问API的认证令牌
- `API_ENDPOINT`: 接收文件上传的API端点URL

## 2. 自定义 Action 配置

打开`.github/workflows/deploy.yml`文件，根据你的需求进行修改：

### 2.1 修改运行频率

默认配置为每天运行一次，可以根据需要调整：

```yaml
on:
  schedule:
    # 每天UTC时间0点运行（北京时间8点）
    - cron: '0 0 * * *'
  # 允许手动触发
  workflow_dispatch:
```

cron表达式格式：`分 时 日 月 周`

### 2.2 选择部署方式

配置文件中提供了三种部署方式，请根据你的服务器环境选择一种并取消注释，同时注释掉其他两种：

#### SCP 方式（最通用）
```yaml
- name: Deploy to server via SCP
  uses: appleboy/scp-action@master
  with:
    host: ${{ secrets.SERVER_HOST }}
    username: ${{ secrets.SERVER_USERNAME }}
    key: ${{ secrets.SSH_PRIVATE_KEY }}
    port: ${{ secrets.SERVER_PORT || '22' }}
    source: simple_edition/data.json
    target: ${{ secrets.DEPLOY_PATH || '/var/www/html' }}
    overwrite: true
```

#### SFTP 方式
```yaml
- name: Deploy to server via SFTP
  uses: wlixcc/SFTP-Deploy-Action@v1.2.4
  with:
    username: ${{ secrets.SERVER_USERNAME }}
    server: ${{ secrets.SERVER_HOST }}
    ssh_private_key: ${{ secrets.SSH_PRIVATE_KEY }}
    local_path: simple_edition/data.json
    remote_path: ${{ secrets.DEPLOY_PATH || '/var/www/html' }}
```

#### API 上传方式
```yaml
- name: Upload data.json to API
  run: |
    curl -X POST \
      -H "Authorization: Bearer ${{ secrets.API_TOKEN }}" \
      -F "file=@simple_edition/data.json" \
      ${{ secrets.API_ENDPOINT }}
```

### 2.3 配置通知（可选）

如果你想在部署成功或失败时收到通知，可以配置Slack通知：

1. 创建Slack Webhook
2. 添加`SLACK_WEBHOOK`到GitHub Secrets
3. 确保通知步骤没有被注释掉

## 3. 手动触发 Action

除了定时运行外，你还可以随时手动触发Action：

1. 进入你的GitHub仓库
2. 点击 Actions 标签
3. 选择 "Generate and Deploy data.json" workflow
4. 点击 "Run workflow" 按钮

## 4. 常见问题排查

### 4.1 部署失败

- 检查GitHub Secrets是否正确配置
- 确认服务器防火墙是否允许SSH连接
- 验证服务器上的目标路径是否存在且有写入权限

### 4.2 文件未更新

- 检查Action日志，确认`data.json`是否成功生成
- 验证服务器上的目标路径是否正确
- 检查是否有其他进程可能在覆盖该文件

### 4.3 SSH连接问题

- 确保私钥格式正确（包含完整的私钥内容，包括`-----BEGIN RSA PRIVATE KEY-----`和`-----END RSA PRIVATE KEY-----`）
- 验证服务器上已配置对应的公钥
- 尝试使用密码认证（需要额外配置`PASSWORD` Secret并修改Action配置）

## 5. 安全性考虑

- 不要在仓库代码中硬编码任何敏感信息
- 定期更新SSH密钥
- 考虑使用只读权限的API令牌
- 限制服务器上的写入权限，仅授予必要目录的权限

## 6. 扩展功能

如果你需要更复杂的部署流程，可以考虑：

- 添加部署前的备份步骤
- 增加部署后的验证检查
- 配置多环境部署（测试/生产）
- 添加条件部署规则

通过正确配置GitHub Action，你可以实现`data.json`文件的自动定期更新和部署，无需手动操作。