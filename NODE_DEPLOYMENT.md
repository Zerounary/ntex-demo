# 游戏节点自动部署脚本使用说明

## 功能特性

1. **XrayR 自动安装和配置**
   - 使用官方一键安装脚本安装 XrayR
   - 自动读取 XrayR 配置文件获取节点信息
   - 自动启动 XrayR 服务

2. **防火墙自动配置**
   - 支持 ufw、firewalld、iptables
   - 自动开放 XrayR API 端口

3. **节点自动注册**
   - 向后台服务器自动注册节点
   - 推送节点配置信息（VMess UUID、端口、服务器IP等）
   - 自动计算网络延迟

4. **定时心跳**
   - 每5分钟自动发送心跳到服务器
   - 保持节点在线状态

## 使用方法

### 1. 基本使用

```bash
# 使用默认配置（服务器地址需要修改）
sudo SERVER_URL=http://your-server.com:8080 ./node-install.sh
```

### 2. 完整配置

```bash
# 设置所有参数
sudo SERVER_URL=http://your-server.com:8080 \
     API_TOKEN=your-api-token \
     NODE_ID=node-custom-id \
     XRAYR_API_PORT=54321 \
     ./node-install.sh
```

### 3. 环境变量说明

| 变量名 | 说明 | 默认值 | 必需 |
|--------|------|--------|------|
| `SERVER_URL` | 后台服务器地址 | `http://localhost:8080` | 是 |
| `API_TOKEN` | API 认证 Token | 空 | 否 |
| `NODE_ID` | 节点ID | 自动生成 | 否 |
| `XRAYR_API_PORT` | XrayR API 端口 | `54321` | 否 |

### 4. XrayR 配置要求

脚本会自动读取 `/etc/XrayR/config.yml` 配置文件，需要确保配置文件中包含：

- **ApiConfig**: API 配置（端口、Token）
- **Nodes**: 节点配置（至少一个节点）
  - **VmessSettings**: VMess 配置
    - **UUID**: VMess UUID
    - **Port**: VMess 端口
    - **Email**: VMess 邮箱

### 5. 示例 XrayR 配置

```yaml
ApiConfig:
  Listen: 0.0.0.0:54321
  Token: your-api-token-here

Nodes:
  - NodeType: V2ray
    VmessSettings:
      UUID: your-uuid-here
      Port: 443
      Email: user@example.com
```

## 安装前准备

### 1. 系统要求

- Linux 系统（支持 Ubuntu、Debian、CentOS 等）
- Root 权限
- 网络连接

### 2. 依赖检查

脚本会自动检测并使用以下工具（如果可用）：
- `yq`: YAML 解析工具（推荐）
- `python3`: Python 3（用于 YAML 解析）
- `curl`: HTTP 请求工具

### 3. 安装 yq（推荐，用于解析 YAML）

```bash
# Ubuntu/Debian
sudo wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
sudo chmod +x /usr/local/bin/yq

# CentOS/RHEL
sudo wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
sudo chmod +x /usr/local/bin/yq
```

## 工作流程

1. **环境检测**
   - 检查 root 权限
   - 检测系统类型
   - 检测防火墙类型

2. **安装 XrayR**
   - 使用官方脚本安装
   - 如果已安装则跳过

3. **读取配置**
   - 读取 XrayR 配置文件
   - 提取节点信息
   - 获取服务器 IP

4. **配置防火墙**
   - 开放 XrayR API 端口
   - 根据防火墙类型选择相应命令

5. **启动服务**
   - 启动 XrayR 服务
   - 验证服务状态

6. **注册节点**
   - 向服务器发送注册请求
   - 包含节点完整信息

7. **设置心跳**
   - 创建心跳脚本
   - 添加到 crontab（每5分钟）

## 验证部署

### 1. 检查 XrayR 服务

```bash
systemctl status XrayR
```

### 2. 检查节点注册

访问后台服务器，查看节点列表：
```
GET /api/nodes
```

### 3. 检查心跳日志

```bash
tail -f /var/log/xrayr-heartbeat.log
```

### 4. 手动发送心跳测试

```bash
/usr/local/bin/xrayr-heartbeat.sh
```

## 故障排查

### 1. XrayR 安装失败

- 检查网络连接
- 检查系统兼容性
- 查看官方文档：https://xrayr-project.github.io/XrayR-doc/

### 2. 配置文件读取失败

- 确保 `/etc/XrayR/config.yml` 存在
- 检查 YAML 格式是否正确
- 安装 `yq` 或确保 `python3` 可用

### 3. 防火墙配置失败

- 检查防火墙服务是否运行
- 手动配置防火墙规则
- 查看防火墙日志

### 4. 节点注册失败

- 检查服务器地址是否正确
- 检查网络连接
- 检查 API Token（如果使用）
- 查看服务器日志

### 5. 心跳失败

- 检查 crontab 是否正确设置：`crontab -l`
- 检查心跳脚本权限：`ls -l /usr/local/bin/xrayr-heartbeat.sh`
- 查看心跳日志：`cat /var/log/xrayr-heartbeat.log`

## 卸载

如果需要卸载节点：

```bash
# 1. 停止 XrayR
systemctl stop XrayR
systemctl disable XrayR

# 2. 删除心跳任务
crontab -l | grep -v xrayr-heartbeat | crontab -

# 3. 删除心跳脚本
rm -f /usr/local/bin/xrayr-heartbeat.sh

# 4. 向服务器注销节点（需要手动调用 API）
curl -X POST http://your-server.com:8080/api/nodes/{node_id}/unregister
```

## 注意事项

1. **安全性**
   - 确保 API Token 安全
   - 定期更新 XrayR
   - 配置防火墙规则

2. **网络要求**
   - 确保服务器可以访问后台 API
   - 确保防火墙允许必要端口

3. **配置备份**
   - 定期备份 XrayR 配置
   - 记录节点 ID 和配置信息

## 相关链接

- XrayR 官方文档：https://xrayr-project.github.io/XrayR-doc/
- XrayR 一键安装：https://raw.githubusercontent.com/XrayR-project/XrayR-release/master/install.sh



