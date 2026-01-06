# /accelerator/session/start_v2 使用与测试指南

## 1. 背景概述

为了解决“一个游戏多线路、线路可绑定真实节点或链路（chain）”的维护问题，新增了 `POST /api/accelerator/session/start_v2` 接口：

- **兼容旧流程**：仍返回 `ProfileVO` 给 Tauri，客户端无需升级即可使用。
- **灵活入口**：请求既可以指定 `nodeId`（直连真实节点），也可以指定 `chainId`（通过链路入口 IP/port 连到真实节点）。
- **链路解析**：当传入 `chainId` 时，服务端会解析 `admin_chains` 的第一跳作为入口、最后一跳作为真实节点，并在返回的 `ProfileVO` 中覆盖 `vmessServer`/`vmessPort` 为链路入口。

## 2. 请求格式

```http
POST /api/accelerator/session/start_v2
Content-Type: application/json
Authorization: Bearer <user-token>

{
  "gameId": "<game_id>",
  "nodeId": 123,            // 任选其一
  "chainId": "hk-route-1", // ---
  "basePort": 40000         // 可选，chain 入口端口，默认 40000
}
```

> **约束**：`nodeId` 与 `chainId` 必须二选一，不能同时出现，也不能同时缺失。

| 字段      | 类型            | 说明 |
|-----------|-----------------|------|
| `gameId`  | `string`        | 目标游戏 ID，与旧接口一致 |
| `nodeId`  | `number (u64)`  | 直接使用真实节点（不经过 chain）时填写 |
| `chainId` | `string`        | 指定链路 ID，需在 `admin_chains` 中存在 |
| `basePort`| `number (u16)`  |（仅 chain 模式）作为链路入口端口，默认 `40000` |

## 3. 响应示例

```json
{
  "msg": "ok",
  "data": {
    "sessionId": "51a7...",
    "uuid": "2fd9...",
    "billType": "minute",
    "remainingMinutes": 42,
    "profile": {
      "id": "profile_1",
      "gameId": "game_1",
      "displayName": "东京-A",
      "nodeId": "12",
      "processName": "cs2.exe",
      "vmessUuid": "2fd9...",
      "vmessServer": "203.0.113.10",    // chain 模式：入口 IP
      "vmessPort": 40000,                // chain 模式：入口端口
      "vmessEmail": "user@example.com",
      "udpProxy": "127.0.0.1:2801",
      "mode": "tcp+udp",
      "status": "active",
      "region": "JP",
      "ping": 36
    }
  }
}
```

> 如果走直连节点（`nodeId`），`vmessServer`/`vmessPort` 保持原节点值，不会覆盖。

## 4. 链路模式解析逻辑

1. 读取 `admin_chains (node_id=0)` 中的链路定义，找到 `chainId` 对应配置。
2. 对 `routes` 按 `order` 排序：
   - 第一条 `from_node_id` => 入口节点（用于获取 `public_ip`）。
   - 最后一条 `to_node_id` => 真实节点（`exitNodeId`）。
3. `entryServer` = 入口节点的 `public_ip`，`entryPort` = `basePort`（默认 40000）。
4. `exitNodeId` 用于：
   - 校验节点在线
   - `main_admin_add_user` / `main_admin_add_mapping`
   - `acceleration_session` 中的 `node_id`
5. 返回的 `ProfileVO`：
   - chain 模式：`vmessServer`/`vmessPort` 覆盖为入口；其余字段来自 `accelerator_nodes`/`accelerator_profiles`（真实节点缓存）。

> 当前实现仅在解析 chain 时 **best-effort 推送一次 MQTT `config` 更新** 给入口节点；如果期望自动执行 `apply_chain`，需要在管理侧确保链路已生效或扩展后端逻辑。

## 5. 测试步骤

### 5.1 准备条件

1. **数据库**：
   - `admin_chains` 中存在目标 `chainId`，且包含至少一条路由。
   - `accelerator_profiles` / `accelerator_nodes` 中存在 `gameId + nodeId(exit)` 对应记录。
2. **节点状态**：
   - `admin_node_configs.node_id = exitNodeId` 处于在线（`is_online=true` 且 `last_seen_at` 未过期）。
3. **账号**：
   - 使用已有 `Bearer` token，且账号有效、余额充足（分钟制必须 `remaining_minutes > 0`）。

### 5.2 cURL 示例

#### A. 直连节点
```bash
curl -X POST http://localhost:8080/api/accelerator/session/start_v2 \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "gameId": "game_1",
    "nodeId": 12
  }'
```

#### B. 链路入口
```bash
curl -X POST http://localhost:8080/api/accelerator/session/start_v2 \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "gameId": "game_1",
    "chainId": "hk-route-1",
    "basePort": 40000
  }'
```

### 5.3 验证要点

1. **HTTP 响应**：
   - `msg` 为 `ok`
   - `profile.vmessServer`/`vmessPort` 是否正确（chain 模式应为入口 IP/port）。
2. **数据库**：
   - `acceleration_sessions` 新增记录，`node_id` 应为 `exitNodeId`。
3. **面板**：
   - 管理端可在对应节点看到新建的 admin user/mapping。
4. **客户端**：
   - Tauri 启动后可正常连通（链路入口端口需在防火墙放行且链路已 apply）。

## 6. 回退/兼容性

- 原 `/accelerator/session/start` 未改动，旧客户端仍可使用。
- 前端/客户端若要启用链路模式，仅需调用新接口并传 `chainId`。
- 后续若计划直接下发完整 `admin_outbounds`/`admin_routings` 配置，可在此文档基础上拓展。
