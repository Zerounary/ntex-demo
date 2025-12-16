# XrayR 本地 API 服务器 - HTTP 测试接口文档

## 概述

本文档描述了本地 MQTT API 服务器的所有 HTTP 接口，用于测试和管理 XrayR 的配置。

**服务器地址**: `http://127.0.0.1:667`

**Content-Type**: `application/json`

---

## 一、查询接口（GET）

### 1. 获取用户列表

**接口路径**: `GET /?act=user`

**描述**: 获取所有用户列表

**请求参数**:
- `act` (query): 固定值 `user`

**请求示例**:
```bash
curl "http://127.0.0.1:667?act=user"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": [
    {
      "id": 1,
      "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "st": 5,
      "dt": 0
    },
    {
      "id": 2,
      "uuid": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
      "st": 1,
      "dt": 0
    }
  ]
}
```

**字段说明**:
- `id`: 用户 ID（数字）
- `uuid`: 用户 UUID（字符串，VMess/V2Ray/VLess 用户必需）
- `st`: 限速值（Mbps，数字）
- `dt`: 设备限制（数字，0 表示不限制）

---

### 2. 获取节点配置

**接口路径**: `GET /?act=config`

**描述**: 获取节点配置信息

**请求参数**:
- `act` (query): 固定值 `config`

**请求示例**:
```bash
curl "http://127.0.0.1:667?act=config"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "node_id": 41,
    "node_type": "Vmess",
    "node_speed_limit": 0,
    "traffic_rate": 1.0,
    "sort": 1,
    "inbounds": [
      {
        "port": 10086,
        "protocol": "vmess",
        "settings": {},
        "streamSettings": {
          "network": "tcp"
        }
      }
    ]
  }
}
```

---

### 3. 获取上游代理配置

**接口路径**: `GET /?act=outbound`

**描述**: 获取所有上游代理配置和用户映射关系

**请求参数**:
- `act` (query): 固定值 `outbound`

**请求示例**:
```bash
curl "http://127.0.0.1:667?act=outbound"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "outbounds": [
      {
        "tag": "ss_1",
        "protocol": "shadowsocks",
        "settings": {
          "servers": [
            {
              "address": "67.209.176.181",
              "port": 19166,
              "method": "aes-256-gcm",
              "password": "bxaeWJ4Kf9ZL59R3"
            }
          ]
        }
      }
    ],
    "user_mapping": {
      "a1b2c3d4-e5f6-7890-abcd-ef1234567890": "ss_1",
      "b2c3d4e5-f6a7-8901-bcde-f12345678901": "ss_2"
    }
  }
}
```

**字段说明**:
- `outbounds`: 上游代理配置数组
  - `tag`: 代理标签（唯一标识）
  - `protocol`: 协议类型（如 `shadowsocks`）
  - `settings`: 代理设置（包含服务器信息）
- `user_mapping`: 用户到上游代理的映射（UUID -> tag）

---

### 4. 获取路由配置

**接口路径**: `GET /?act=routing`

**描述**: 获取 Xray 路由规则配置

**请求参数**:
- `act` (query): 固定值 `routing`

**请求示例**:
```bash
curl "http://127.0.0.1:667?act=routing"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "domainStrategy": "IPOnDemand",
    "rules": [
      {
        "type": "field",
        "outboundTag": "block",
        "domain": [
          "geosite:gfw",
          "geosite:greatfire"
        ]
      },
      {
        "type": "field",
        "outboundTag": "block",
        "ip": [
          "geoip:cloudflare",
          "geoip:google"
        ]
      }
    ]
  }
}
```

**字段说明**:
- `domainStrategy`: 域名解析策略（如 `IPOnDemand`、`AsIs`、`IPIfNonMatch`、`IPv4Only`、`IPv6Only`）
- `rules`: 路由规则数组
  - `type`: 规则类型（固定为 `field`）
  - `outboundTag`: 匹配时使用的 outbound 标签
  - `domain`: 域名匹配规则（支持 `geosite:`、`regexp:`、`full:`、`keyword:`、`domain:` 等）
  - `ip`: IP 匹配规则（支持 `geoip:`、CIDR 格式等）
  - `port`: 端口匹配规则
  - `network`: 网络类型匹配（`tcp`、`udp`、`tcp,udp`）
  - `source`: 源 IP 匹配规则
  - `protocol`: 协议匹配规则（如 `http`、`tls`、`bittorrent` 等）

**注意事项**:
- 使用 `geosite:` 和 `geoip:` 规则需要确保 `geosite.dat` 和 `geoip.dat` 文件在 XrayR 运行目录
- 路由配置是 Core 级别的全局配置，修改后需要重启 XrayR 才能生效

---

### 5. 查询用户日志

**接口路径**: `GET /?act=user_logs`

**描述**: 通过 MQTT 从 XrayR 节点查询指定用户的日志

**请求参数**:
- `act` (query): 固定值 `user_logs`
- `uid` (query, 必需): 用户 ID
- `days` (query, 可选): 查询最近 N 天的日志，默认为 7
- `event` (query, 可选): 事件类型过滤（如：`connection_new`、`speed_limit`、`device_limit` 等），空字符串表示不过滤
- `limit` (query, 可选): 返回结果数量限制，默认为 100，0 表示不限制

**请求示例**:
```bash
# 查询用户 1 最近 7 天的日志
curl "http://127.0.0.1:667?act=user_logs&uid=1"

# 查询用户 1 最近 3 天的连接事件日志，最多返回 50 条
curl "http://127.0.0.1:667?act=user_logs&uid=1&days=3&event=connection_new&limit=50"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "entries": [
      {
        "timestamp": "2025-12-12T10:30:45Z",
        "uid": 1,
        "email": "a1b2c3d4-e5f6-7890-abcd-ef1234567890@x.com",
        "ip": "192.168.1.100",
        "tag": "inbound-0",
        "event": "connection_new",
        "message": "User connection: a1b2c3d4-e5f6-7890-abcd-ef1234567890@x.com (IP: 192.168.1.100)",
        "data": {
          "speed_limit_mbps": 5.0,
          "is_new": true
        }
      },
      {
        "timestamp": "2025-12-12T10:25:30Z",
        "uid": 1,
        "email": "a1b2c3d4-e5f6-7890-abcd-ef1234567890@x.com",
        "ip": "192.168.1.100",
        "tag": "inbound-0",
        "event": "speed_limit",
        "message": "Speed limit applied: 5.00 Mbps (per-user mode)",
        "data": {
          "speed_limit_mbps": 5.0,
          "mode": "per-user"
        }
      }
    ],
    "total": 2
  }
}
```

**字段说明**:
- `entries`: 日志条目数组
  - `timestamp`: 日志时间戳（ISO 8601 格式）
  - `uid`: 用户 ID
  - `email`: 用户邮箱（可选）
  - `ip`: 用户 IP 地址（可选）
  - `tag`: 连接标签（可选）
  - `event`: 事件类型（如：`connection_new`、`connection_reuse`、`speed_limit`、`device_limit`、`routing_match` 等）
  - `message`: 日志消息
  - `data`: 附加数据（可选，包含事件相关的详细信息）
- `total`: 匹配的日志总数（可能大于返回的条目数，如果设置了 limit）

**错误响应**:
```json
{
  "msg": "error",
  "error": "uid 参数是必需的"
}
```

或者

```json
{
  "msg": "error",
  "error": "MQTT 未连接，无法查询日志"
}
```

或者

```json
{
  "msg": "error",
  "error": "查询日志超时或失败"
}
```

**HTTP 状态码**:
- `200`: 查询成功
- `400`: 请求参数错误
- `503`: MQTT 未连接
- `504`: 查询超时

**注意事项**:
- 此接口通过 MQTT 从 XrayR 节点实时查询日志，需要确保 MQTT 连接正常
- 查询超时时间为 15 秒
- 日志查询需要节点启用用户日志功能（配置 `Log.UserLogDir`）
- 日志按时间倒序返回（最新的在前）
- 支持的事件类型包括：
  - `connection_new`: 新连接建立
  - `connection_reuse`: 复用现有连接
  - `speed_limit`: 限速事件
  - `device_limit`: 设备限制事件
  - `routing_match`: 路由匹配事件
  - `routing_debug`: 路由调试信息
  - 其他自定义事件类型

---

## 二、管理接口 - 用户管理

### 6. 添加用户

**接口路径**: `POST /api/admin/user`

**描述**: 添加新用户

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "uuid": "new-uuid-here",
  "st": 10,
  "dt": 0
}
```

**字段说明**:
- `uuid` (必需): 用户 UUID
- `st` (可选): 限速值（Mbps），默认为 1
- `dt` (可选): 设备限制，默认为 0

**请求示例**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/user \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "new-uuid-1234-5678-90ab-cdef12345678",
    "st": 10,
    "dt": 0
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "id": 7,
    "uuid": "new-uuid-1234-5678-90ab-cdef12345678",
    "st": 10,
    "dt": 0
  }
}
```

**注意事项**:
- 用户 ID 会自动生成（当前最大 ID + 1）
- 添加成功后会自动通过 MQTT 推送更新通知
- XrayR 节点会立即收到通知并更新配置

---

### 7. 更新用户

**接口路径**: `PUT /api/admin/user/{user_id}`

**描述**: 更新指定用户的信息

**路径参数**:
- `user_id`: 用户 ID（数字）

**请求头**:
```
Content-Type: application/json
```

**请求体**（所有字段可选）:
```json
{
  "uuid": "updated-uuid",
  "st": 20,
  "dt": 2
}
```

**请求示例**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/user/1 \
  -H "Content-Type: application/json" \
  -d '{
    "st": 20,
    "dt": 2
  }'
```

**响应示例**:
```json
{
  "msg": "ok"
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "用户不存在"
}
```

**HTTP 状态码**:
- `200`: 更新成功
- `400`: 请求参数错误
- `404`: 用户不存在

---

### 8. 删除用户

**接口路径**: `DELETE /api/admin/user/{user_id}`

**描述**: 删除指定用户

**路径参数**:
- `user_id`: 用户 ID（数字）

**请求示例**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/user/1
```

**响应示例**:
```json
{
  "msg": "ok"
}
```

**注意事项**:
- 删除用户时会同时删除该用户的所有映射关系
- 删除成功后会自动推送用户和上游代理更新通知

---

## 三、管理接口 - 上游代理管理

### 9. 添加上游代理

**接口路径**: `POST /api/admin/outbound`

**描述**: 添加新的上游代理配置

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "tag": "ss_new",
  "protocol": "shadowsocks",
  "settings": {
    "servers": [
      {
        "address": "1.2.3.4",
        "port": 8388,
        "method": "aes-256-gcm",
        "password": "password123"
      }
    ]
  }
}
```

**字段说明**:
- `tag` (必需): 代理标签（唯一标识）
- `protocol` (可选): 协议类型，默认为 `shadowsocks`
- `settings` (可选): 代理设置，默认为空对象

**请求示例**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/outbound \
  -H "Content-Type: application/json" \
  -d '{
    "tag": "ss_new",
    "protocol": "shadowsocks",
    "settings": {
      "servers": [
        {
          "address": "1.2.3.4",
          "port": 8388,
          "method": "aes-256-gcm",
          "password": "password123"
        }
      ]
    }
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "tag": "ss_new",
    "protocol": "shadowsocks",
    "settings": {
      "servers": [
        {
          "address": "1.2.3.4",
          "port": 8388,
          "method": "aes-256-gcm",
          "password": "password123"
        }
      ]
    }
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "tag \"ss_new\" 已存在"
}
```

**注意事项**:
- `tag` 必须唯一，不能与现有代理标签重复
- 添加成功后会自动通过 MQTT 推送更新通知

---

### 10. 更新上游代理

**接口路径**: `PUT /api/admin/outbound/{tag}`

**描述**: 更新指定上游代理的配置

**路径参数**:
- `tag`: 代理标签

**请求头**:
```
Content-Type: application/json
```

**请求体**（所有字段可选）:
```json
{
  "protocol": "shadowsocks",
  "settings": {
    "servers": [
      {
        "address": "5.6.7.8",
        "port": 8888,
        "method": "chacha20-poly1305",
        "password": "newpassword"
      }
    ]
  }
}
```

**请求示例**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/outbound/ss_1 \
  -H "Content-Type: application/json" \
  -d '{
    "settings": {
      "servers": [
        {
          "address": "5.6.7.8",
          "port": 8888,
          "method": "chacha20-poly1305",
          "password": "newpassword"
        }
      ]
    }
  }'
```

**响应示例**:
```json
{
  "msg": "ok"
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "上游代理不存在"
}
```

---

### 10.1 查询指定上游的 UDP 延迟（主动）

**接口路径**: `GET /api/admin/outbound/udp_latency`

**描述**: 通过 MQTT 主动向节点请求指定 outbound 的 UDP 延迟

**请求参数（query）**:
- `outbound_tag` (必需): 要测试的上游标签
- `node_id` (可选): 目标节点 ID，默认使用当前节点

**请求示例**:
```bash
curl "http://127.0.0.1:667/api/admin/outbound/udp_latency?outbound_tag=ss_1"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "outbound_tag": "ss_1",
    "latency_ms": 42.8,
    "timestamp": 1702454600
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "MQTT 未连接，无法查询 UDP 延迟"
}
```

---

### 11. 删除上游代理

**接口路径**: `DELETE /api/admin/outbound/{tag}`

**描述**: 删除指定的上游代理

**路径参数**:
- `tag`: 代理标签

**请求示例**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/outbound/ss_1
```

**响应示例**:
```json
{
  "msg": "ok"
}
```

**注意事项**:
- 删除代理时会自动删除所有使用该代理的用户映射
- 删除成功后会自动推送更新通知

---

## 四、管理接口 - 路由配置管理

### 12. 更新整个路由配置

**接口路径**: `POST /api/admin/routing`

**描述**: 更新整个路由配置（替换所有规则）

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "domainStrategy": "IPOnDemand",
  "rules": [
    {
      "type": "field",
      "outboundTag": "block",
      "domain": [
        "geosite:gfw"
      ]
    }
  ]
}
```

**字段说明**:
- `domainStrategy` (可选): 域名解析策略
- `rules` (可选): 路由规则数组
- `routing` (可选): 完整的路由配置对象（如果提供，会直接替换整个配置）

**请求示例**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/routing \
  -H "Content-Type: application/json" \
  -d '{
    "domainStrategy": "IPOnDemand",
    "rules": [
      {
        "type": "field",
        "outboundTag": "block",
        "domain": ["geosite:gfw"]
      }
    ]
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "domainStrategy": "IPOnDemand",
    "rules": [
      {
        "type": "field",
        "outboundTag": "block",
        "domain": ["geosite:gfw"]
      }
    ]
  }
}
```

**注意事项**:
- 更新成功后会自动通过 MQTT 推送更新通知
- 路由配置修改后需要重启 XrayR 才能生效（Core 级别配置）

---

### 13. 添加路由规则

**接口路径**: `POST /api/admin/routing/rule`

**描述**: 在现有路由配置中添加一条新的路由规则

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "type": "field",
  "outboundTag": "block",
  "domain": [
    "regexp:.*\\.google\\.com$"
  ]
}
```

**字段说明**:
- `type` (必需): 规则类型，固定为 `field`
- `outboundTag` (必需): 匹配时使用的 outbound 标签
- `domain` (可选): 域名匹配规则数组
- `ip` (可选): IP 匹配规则数组
- `port` (可选): 端口匹配规则
- `network` (可选): 网络类型匹配
- `source` (可选): 源 IP 匹配规则
- `protocol` (可选): 协议匹配规则

**请求示例**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/routing/rule \
  -H "Content-Type: application/json" \
  -d '{
    "type": "field",
    "outboundTag": "block",
    "domain": ["regexp:.*\\.youtube\\.com$"]
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "domainStrategy": "IPOnDemand",
    "rules": [
      {
        "type": "field",
        "outboundTag": "block",
        "domain": ["geosite:gfw"]
      },
      {
        "type": "field",
        "outboundTag": "block",
        "domain": ["regexp:.*\\.youtube\\.com$"]
      }
    ]
  }
}
```

**注意事项**:
- 新规则会追加到现有规则列表的末尾
- 规则匹配按顺序进行，第一个匹配的规则会被应用
- 添加成功后会自动推送更新通知

---

### 14. 更新路由规则

**接口路径**: `PUT /api/admin/routing/rule/{index}`

**描述**: 更新指定索引位置的路由规则

**路径参数**:
- `index`: 规则索引（从 0 开始）

**请求头**:
```
Content-Type: application/json
```

**请求体**（所有字段可选）:
```json
{
  "outboundTag": "direct",
  "domain": [
    "geosite:cn"
  ]
}
```

**请求示例**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/routing/rule/0 \
  -H "Content-Type: application/json" \
  -d '{
    "outboundTag": "direct",
    "domain": ["geosite:cn"]
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "type": "field",
    "outboundTag": "direct",
    "domain": ["geosite:cn"]
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "规则索引超出范围"
}
```

**HTTP 状态码**:
- `200`: 更新成功
- `400`: 请求参数错误
- `404`: 规则索引超出范围

**注意事项**:
- 只能更新已存在的规则，不能改变规则类型
- 更新成功后会自动推送更新通知

---

### 15. 删除路由规则

**接口路径**: `DELETE /api/admin/routing/rule/{index}`

**描述**: 删除指定索引位置的路由规则

**路径参数**:
- `index`: 规则索引（从 0 开始）

**请求示例**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/routing/rule/0
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "deleted_rule": {
      "type": "field",
      "outboundTag": "block",
      "domain": ["geosite:gfw"]
    }
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "规则索引超出范围"
}
```

**注意事项**:
- 删除规则后，后续规则的索引会发生变化
- 删除成功后会自动推送更新通知

---

## 五、管理接口 - 用户映射管理

### 16. 添加用户映射

**接口路径**: `POST /api/admin/mapping`

**描述**: 添加用户到上游代理的映射关系

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "outbound_tag": "ss_1"
}
```

**字段说明**:
- `uuid` (必需): 用户 UUID
- `outbound_tag` (必需): 上游代理标签

**请求示例**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/mapping \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "outbound_tag": "ss_1"
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "a1b2c3d4-e5f6-7890-abcd-ef1234567890": "ss_1"
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "uuid 和 outbound_tag 都是必需的"
}
```

**注意事项**:
- 如果映射已存在，会被新值覆盖
- 添加成功后会自动推送更新通知

---

### 17. 更新用户映射

**接口路径**: `PUT /api/admin/mapping/{uuid}`

**描述**: 更新指定用户的映射关系

**路径参数**:
- `uuid`: 用户 UUID

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "outbound_tag": "ss_2"
}
```

**字段说明**:
- `outbound_tag` (必需): 新的上游代理标签

**请求示例**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/mapping/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Content-Type: application/json" \
  -d '{
    "outbound_tag": "ss_2"
  }'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "old_outbound_tag": "ss_1",
    "new_outbound_tag": "ss_2"
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "映射不存在"
}
```

或者

```json
{
  "msg": "error",
  "error": "outbound_tag 是必需的"
}
```

**注意事项**:
- 只能更新已存在的映射，如果映射不存在会返回 404 错误
- 更新成功后会自动推送更新通知
- XrayR 节点会立即收到通知并更新配置，无需等待定时周期

---

### 18. 删除用户映射

**接口路径**: `DELETE /api/admin/mapping/{uuid}`

**描述**: 删除指定用户的映射关系

**路径参数**:
- `uuid`: 用户 UUID

**请求示例**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/mapping/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**响应示例**:
```json
{
  "msg": "ok"
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "映射不存在"
}
```

---

## 六、兼容接口（旧版 API 路径）

### 19. 获取用户列表（兼容路径）

**接口路径**: `GET /api/v1/server/UniProxy/user`

**描述**: 与 `GET /?act=user` 功能相同

---

### 20. 获取节点配置（兼容路径）

**接口路径**: `GET /api/v1/server/UniProxy/config`

**描述**: 与 `GET /?act=config` 功能相同

---

### 21. 获取上游代理配置（兼容路径）

**接口路径**: `GET /api/v1/server/UniProxy/outbound`

**描述**: 与 `GET /?act=outbound` 功能相同

---

### 22. 获取路由配置（兼容路径）

**接口路径**: `GET /api/v1/server/UniProxy/routing`

**描述**: 与 `GET /?act=routing` 功能相同

---

### 23. 流量上报（兼容路径）

### 24. 查询用户日志（兼容路径）

**接口路径**: `GET /api/v1/server/UniProxy/user_logs`

**描述**: 与 `GET /?act=user_logs` 功能相同

---

**接口路径**: `POST /api/v1/server/UniProxy/push`

**描述**: 接收流量上报数据（当前仅返回成功，不处理数据）

**响应示例**:
```json
{
  "msg": "ok"
}
```

---

## 七、错误响应格式

所有接口在出错时都会返回统一格式的错误响应：

```json
{
  "msg": "error",
  "error": "错误描述信息"
}
```

**常见 HTTP 状态码**:
- `200`: 请求成功
- `400`: 请求参数错误（如缺少必需字段、JSON 格式错误等）
- `404`: 资源不存在（如用户不存在、代理不存在等）

---

## 八、MQTT 更新通知机制

当通过管理接口修改配置后，服务器会自动通过 MQTT 推送更新通知：

- **主题格式**: `xrayr/node/{node_id}/update/{type}`
- **通知类型**:
  - `user`: 用户列表更新
  - `outbound`: 上游代理配置更新
  - `config`: 节点配置更新
  - `routing`: 路由配置更新

**通知消息格式**:
```json
{
  "type": "user",
  "timestamp": 1234567890.123,
  "action": "update"
}
```

XrayR 节点收到通知后会立即重新拉取配置，无需等待定时周期（默认 60 秒）。

---

## 九、MQTT 上报机制

XrayR 节点会通过 MQTT 主动上报各种状态信息：

### 1. 流量上报 (submit)

**操作**: `submit`

**上报内容**: 用户流量统计

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "submit",
  "data": [
    {
      "uid": 1,
      "u": 1024000,
      "d": 2048000
    }
  ]
}
```

### 2. 节点状态上报 (nodestatus)

**操作**: `nodestatus`

**上报内容**: 节点系统资源使用情况

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "nodestatus",
  "data": {
    "cpu": "50%",
    "mem": "60%",
    "disk": "70%",
    "uptime": 3600
  }
}
```

### 3. 在线用户上报 (onlineusers)

**操作**: `onlineusers`

**上报内容**: 当前在线用户列表

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "onlineusers",
  "data": [
    {
      "uid": 1,
      "ip": "192.168.1.100"
    }
  ]
}
```

### 4. 非法行为上报 (illegal)

**操作**: `illegal`

**上报内容**: 用户触发的非法行为记录

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "illegal",
  "data": [
    {
      "uid": 1
    }
  ]
}
```

### 5. Outbound 连接失败上报 (outbound_failure) ⭐ 新增

**操作**: `outbound_failure`

**上报内容**: 上游代理连接失败信息

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_failure",
  "data": {
    "outbound_tag": "ss_1",
    "error": "connection refused",
    "timestamp": 1702454400,
    "config": {
      "protocol": "shadowsocks",
      "settings": {
        "servers": [{
          "address": "1.2.3.4",
          "port": 8388,
          "method": "aes-256-gcm"
        }]
      }
    }
  }
}
```

**字段说明**:
- `outbound_tag`: 上游代理标签（如 `ss_1`、`ss_2`）
- `error`: 错误信息（如 `connection refused`、`connection timeout` 等）
- `timestamp`: 错误发生的时间戳（Unix 时间戳）
- `config`: 上游代理配置信息（可选，从节点上报）

**触发条件**:
- 当 XrayR 检测到上游代理连接失败时自动上报
- 仅上报真正的连接错误（排除正常的连接关闭）
- 通过包装 `transport.Link` 实时监控连接状态

**服务器端处理**:
服务器收到上报后会：
1. 打印错误信息到控制台
2. 可以扩展为记录到数据库
3. 可以触发告警通知

**示例输出**:
```
🔴 [Outbound 连接失败] Tag: ss_1, Error: connection refused, Time: 2025-12-13 12:30:45
```

---

### 6. Outbound 连接恢复上报 (outbound_recovery) ⭐ 新增

**操作**: `outbound_recovery`

**上报内容**: 上游代理连接恢复信息

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_recovery",
  "data": {
    "outbound_tag": "ss_1",
    "timestamp": 1702454500,
    "config": {
      "protocol": "shadowsocks",
      "settings": {
        "servers": [{
          "address": "1.2.3.4",
          "port": 8388,
          "method": "aes-256-gcm"
        }]
      }
    }
  }
}
```

**字段说明**:
- `outbound_tag`: 上游代理标签
- `timestamp`: 恢复时间戳（Unix 时间戳）
- `config`: 上游代理配置信息（可选，从节点上报）

**触发条件**:
- 当之前失败的连接恢复正常时自动上报
- 需要经过验证期（3秒）确认连接稳定后才上报
- 避免误报：只有失败超过5秒后的恢复才会上报

---

### 7. Outbound 延迟上报 (outbound_latency) ⭐ 新增（单个）

**操作**: `outbound_latency`

**上报内容**: 单个上游代理的延迟测量结果

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_latency",
  "data": {
    "outbound_tag": "ss_1",
    "latency_ms": 125.5,
    "timestamp": 1702454600
  }
}
```

**字段说明**:
- `outbound_tag`: 上游代理标签
- `latency_ms`: 延迟值（毫秒，浮点数）
- `timestamp`: 测量时间戳（Unix 时间戳）

**触发条件**:
- 基于实际用户流量的被动测量
- 每60秒或每100个样本上报一次
- 使用指数移动平均（EMA）算法平滑延迟值

**注意**: 此接口已保留用于兼容性，但推荐使用批量上报接口 `outbound_latencies`。

---

### 8. Outbound 批量延迟上报 (outbound_latencies) ⭐ 新增（批量）

**操作**: `outbound_latencies`

**上报内容**: 多个上游代理的延迟测量结果（批量上报）

**消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_latencies",
  "data": {
    "latencies": [
      {
        "outbound_tag": "ss_1",
        "latency_ms": 125.5,
        "timestamp": 1702454600
      },
      {
        "outbound_tag": "ss_2",
        "latency_ms": 89.3,
        "timestamp": 1702454600
      },
      {
        "outbound_tag": "ss_3",
        "latency_ms": 156.7,
        "timestamp": 1702454600
      }
    ],
    "timestamp": 1702454600
  }
}
```

**字段说明**:
- `latencies`: 延迟测量结果数组
  - `outbound_tag`: 上游代理标签
  - `latency_ms`: 延迟值（毫秒，浮点数）
  - `timestamp`: 测量时间戳（Unix 时间戳）
- `timestamp`: 批量上报的时间戳

**触发条件**:
- 主动探测：每60秒对所有 outbound 进行主动探测
- 被动测量：基于实际用户流量的延迟测量
- 批量上报：收集所有需要上报的延迟数据，一次性批量上报
- 上报频率：每60秒或每100个样本上报一次

**优势**:
- **性能优化**: 减少网络开销（从 N 条消息减少到 1 条）
- **减少序列化**: 只序列化一次，降低 CPU 开销
- **数据一致性**: 所有 outbound 的延迟数据在同一时间点上报
- **服务器友好**: 服务器可以一次性处理所有延迟数据

**服务器端处理**:
服务器收到批量上报后会：
1. 按延迟值排序显示
2. 使用颜色标识延迟等级（🟢 < 50ms, 🟡 < 100ms, 🟠 < 200ms, 🔴 >= 200ms）
3. 可以扩展为记录到数据库或用于监控告警

**示例输出**:
```
📊📊📊  OUTBOUND 批量延迟上报  📊📊📊
================================================================================
  📦 上报数量:      3 个 outbound
  🕐 上报时间:      2025-12-13 16:39:35
  📡 节点 ID:       41

  1. 🟡 ss_2                  -    89.30 ms
  2. 🟡 ss_1                  -   125.50 ms
  3. 🟠 ss_3                  -   156.70 ms
================================================================================
```

---

### 9. Outbound UDP 延迟上报 (outbound_udp_latency / outbound_udp_latencies) ⭐ 新增

**操作**: `outbound_udp_latency`（单个） / `outbound_udp_latencies`（批量）

**单个上报消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_udp_latency",
  "data": {
    "outbound_tag": "ss_1",
    "latency_ms": 85.3,
    "timestamp": 1702454800
  }
}
```

**批量上报消息格式**:
```json
{
  "node_id": 41,
  "token": "123",
  "action": "outbound_udp_latencies",
  "data": {
    "latencies": [
      {"outbound_tag": "ss_1", "latency_ms": 85.3, "timestamp": 1702454800},
      {"outbound_tag": "ss_2", "latency_ms": 112.7, "timestamp": 1702454800}
    ],
    "timestamp": 1702454800
  }
}
```

**说明**:
- 周期性被动/主动 UDP 探测都会通过上述动作上报
- 颜色等级同 TCP：🟢 < 50ms，🟡 < 100ms，🟠 < 200ms，🔴 ≥ 200ms
- 服务器可通过接口 `/api/admin/outbound/udp_latency` 进行按需主动探测

---

## 九、测试示例

### 完整流程示例

1. **查询当前用户列表**:
```bash
curl "http://127.0.0.1:667?act=user"
```

2. **添加新用户**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/user \
  -H "Content-Type: application/json" \
  -d '{"uuid": "test-uuid-001", "st": 5, "dt": 0}'
```

3. **添加上游代理**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/outbound \
  -H "Content-Type: application/json" \
  -d '{
    "tag": "test_ss",
    "protocol": "shadowsocks",
    "settings": {
      "servers": [{
        "address": "test.example.com",
        "port": 8388,
        "method": "aes-256-gcm",
        "password": "test123"
      }]
    }
  }'
```

4. **为用户分配代理**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/mapping \
  -H "Content-Type: application/json" \
  -d '{"uuid": "test-uuid-001", "outbound_tag": "test_ss"}'
```

5. **更新用户限速**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/user/1 \
  -H "Content-Type: application/json" \
  -d '{"st": 10}'
```

6. **更新用户映射**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/mapping/test-uuid-001 \
  -H "Content-Type: application/json" \
  -d '{"outbound_tag": "ss_2"}'
```

7. **删除映射关系**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/mapping/test-uuid-001
```

8. **删除用户**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/user/1
```

9. **获取路由配置**:
```bash
curl "http://127.0.0.1:667?act=routing"
```

10. **更新路由配置**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/routing \
  -H "Content-Type: application/json" \
  -d '{
    "domainStrategy": "IPOnDemand",
    "rules": [
      {
        "type": "field",
        "outboundTag": "block",
        "domain": ["geosite:gfw"]
      }
    ]
  }'
```

13. **添加路由规则**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/routing/rule \
  -H "Content-Type: application/json" \
  -d '{
    "type": "field",
    "outboundTag": "block",
    "domain": ["regexp:.*\\.youtube\\.com$"]
  }'
```

14. **更新路由规则**:
```bash
curl -X PUT http://127.0.0.1:667/api/admin/routing/rule/0 \
  -H "Content-Type: application/json" \
  -d '{
    "outboundTag": "direct"
  }'
```

15. **删除路由规则**:
```bash
curl -X DELETE http://127.0.0.1:667/api/admin/routing/rule/0
```

16. **获取维护模式状态**:
```bash
curl "http://127.0.0.1:667/api/admin/maintenance"
```

17. **启用维护模式**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/maintenance \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

18. **禁用维护模式**:
```bash
curl -X POST http://127.0.0.1:667/api/admin/maintenance \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

---

## 十、管理接口 - 维护模式管理

### 25. 获取维护模式状态

**接口路径**: `GET /api/admin/maintenance` 或 `GET /?act=maintenance`

**描述**: 获取当前维护模式状态

**请求示例**:
```bash
curl "http://127.0.0.1:667/api/admin/maintenance"
```

或

```bash
curl "http://127.0.0.1:667?act=maintenance"
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "maintenance_mode": false,
    "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
  }
}
```

**字段说明**:
- `maintenance_mode`: 维护模式状态（`true` 表示启用，`false` 表示禁用）
- `description`: 维护模式功能说明

---

### 26. 设置维护模式

**接口路径**: `POST /api/admin/maintenance`

**描述**: 启用或禁用维护模式。维护模式开启时：
- 跳过新用户的添加（在 `updateUsersImmediately()` 中检查）
- 保留现有用户，允许已存在用户继续连接
- 在限流器中拦截新用户认证，仅对已存在用户放行

**请求头**:
```
Content-Type: application/json
```

**请求体**:
```json
{
  "enabled": true
}
```

**字段说明**:
- `enabled` (必需): `true` 表示启用维护模式，`false` 表示禁用

**请求示例**:
```bash
# 启用维护模式
curl -X POST http://127.0.0.1:667/api/admin/maintenance \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# 禁用维护模式
curl -X POST http://127.0.0.1:667/api/admin/maintenance \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

**响应示例**:
```json
{
  "msg": "ok",
  "data": {
    "maintenance_mode": true,
    "previous_mode": false,
    "message": "维护模式已启用"
  }
}
```

**错误响应**:
```json
{
  "msg": "error",
  "error": "无效的 JSON 数据"
}
```

**HTTP 状态码**:
- `200`: 设置成功
- `400`: 请求参数错误

**注意事项**:
- 维护模式开启时，尝试添加新用户会返回 503 错误
- 维护模式开启时，已存在用户可以正常连接和使用服务
- 维护模式在 XrayR 节点端通过配置 `ControllerConfig.MaintenanceMode` 设置，也可以通过此接口在测试服务器中模拟
- 维护模式状态会同步到限流器，新用户连接会被拒绝

**维护模式行为说明**:
1. **用户添加层面**: 在 `updateUsersImmediately()` 和 `addNewUser()` 中检查维护模式，跳过新用户添加
2. **用户认证层面**: 在限流器的 `GetUserBucket()` 中检查维护模式，如果用户不在已存在用户列表中，直接拒绝连接
3. **现有用户**: 维护模式不影响已存在用户的正常使用

---

## 十一、注意事项

1. **线程安全**: 所有管理操作都使用了线程锁，保证并发安全
2. **立即生效**: 配置修改后会自动推送 MQTT 通知，XrayR 节点会立即更新
3. **ID 自动生成**: 添加用户时，ID 会自动生成（当前最大 ID + 1）
4. **级联删除**: 
   - 删除用户时会同时删除其映射关系
   - 删除上游代理时会删除所有使用该代理的映射
5. **UUID 格式**: UUID 应使用标准格式（如：`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`）
6. **路由配置限制**:
   - 路由配置是 Core 级别的全局配置，修改后需要重启 XrayR 才能生效
   - 使用 `geosite:` 和 `geoip:` 规则需要确保 `geosite.dat` 和 `geoip.dat` 文件在 XrayR 运行目录
   - 路由规则按顺序匹配，第一个匹配的规则会被应用

---

## 十二、开发建议

### 使用 Postman/Insomnia 测试

可以导入以下 JSON 配置来快速测试所有接口：

```json
{
  "info": {
    "name": "XrayR Local API",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "Get Users",
      "request": {
        "method": "GET",
        "url": "http://127.0.0.1:667?act=user"
      }
    },
    {
      "name": "Add User",
      "request": {
        "method": "POST",
        "header": [{"key": "Content-Type", "value": "application/json"}],
        "body": {
          "mode": "raw",
          "raw": "{\"uuid\": \"test-uuid\", \"st\": 5, \"dt\": 0}"
        },
        "url": "http://127.0.0.1:667/api/admin/user"
      }
    }
  ]
}
```

### 使用 Python requests 库

```python
import requests

BASE_URL = "http://127.0.0.1:667"

# 获取用户列表
response = requests.get(f"{BASE_URL}?act=user")
users = response.json()["data"]

# 添加用户
new_user = {
    "uuid": "test-uuid-001",
    "st": 10,
    "dt": 0
}
response = requests.post(f"{BASE_URL}/api/admin/user", json=new_user)
print(response.json())
```

---

**文档版本**: v1.1  
**最后更新**: 2024年

**更新内容**:
- 新增维护模式管理接口（获取状态、设置状态）
- 维护模式功能说明：开启时跳过新用户添加，仅允许已存在用户连接

