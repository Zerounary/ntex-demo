# 游戏加速器计费系统说明（README）

本文档说明当前仓库中“计费系统”的整体逻辑、数据模型、接口列表，以及客户端如何完成：

- 年卡 / 月卡 / 日卡（时长型）购买/兑换/校验/开始加速/结束加速
- 分钟计费（用量型）购买/兑换/校验/开始加速/结束加速

> 说明：当前实现采用 **`client_api` 与 `main.rs` 分离部署、共享同一数据库** 的架构。
> 
> - `client_api` 负责对客户端提供 Web API（会话 start/stop、CDK 兑换、账号校验等）。
> - `main.rs` 负责 Admin/MQTT 通路（节点侧 XrayR 通过 MQTT 获取配置并重载）。
> 
> **`client_api` 不通过 admin HTTP 调用**，而是直接写 DB（通过 `AdminConfigStore`）完成“下发用户到节点”的数据准备。

---

## 1. 核心概念

### 1.1 两类计费产品

- **时长型（会员）**：日卡 / 月卡 / 年卡
  - 兑换后增加用户的 `valid_until`（到期时间）。
  - 未到期时：允许无限次开始会话（不消耗分钟）。

- **用量型（分钟包）**：分钟计费
  - 兑换后增加用户钱包 `remaining_minutes`。
  - 会话 active 时：按分钟扣减（服务端扣费）。

### 1.2 计费优先级（billing_mode）

当用户开始加速（start session）时：

1. 若 `now < valid_until`：
   - `billing_mode = pass`（按会员放行，不扣分钟）
2. 否则若 `remaining_minutes > 0`：
   - `billing_mode = minute`（允许开始，会话期间按分钟扣）
3. 否则：
   - `billing_mode = none`（拒绝开始）

该策略由 `validate_account` 返回的字段体现：

- `remaining_minutes`
- `billing_mode`（`pass | minute | none`）

---

## 2. 部署与数据面（重要）

### 2.1 进程职责

- **client_api（Web 网关/客户端连接服务）**
  - 入口：`src/bin/client_api.rs`
  - 路由：`src/interface/web/routes.rs`
  - 处理器：`src/interface/web/handlers.rs`
  - 依赖：共享 DB（`DATABASE_URL`）

- **main.rs（管理服务 + MQTT 通路）**
  - 入口：`src/main.rs`
  - 管理服务：`src/interface/admin/*`
  - 管理配置存储（写 DB）：`src/infrastructure/admin_config.rs`
  - MQTT：`src/infrastructure/mqtt_client.rs`

### 2.2 “下发用户到节点”如何发生（DB → MQTT → 节点）

会话开始时，`client_api` 会通过 `AdminConfigStore` **直接写 DB**：

- 写入 `admin_user`：创建本次会话的 `uuid`
- 写入 `admin_user_mapping`：将该 `uuid` 绑定到 `outbound_tag`
- 可选：写入链路相关 inbounds（`chain_ops::apply_chain`）

随后由 `main.rs` 的 MQTT 通路通知/驱动节点侧更新（节点侧 XrayR 通过 MQTT 拉取配置并热更新）。

---

## 3. 数据库表（计费相关）

### 3.1 会员到期（已有）

- 表：`accelerator_users`
- 字段：`valid_until`

用于日卡/月卡/年卡的“到期放行”。

### 3.2 分钟钱包（新增）

- 表：`user_wallets`
- 对应实体：`src/infrastructure/persistence/user_wallet.rs`

字段：

- `user_id`（PK）
- `remaining_minutes`（剩余分钟数）
- `updated_at`

### 3.3 加速会话（新增）

- 表：`acceleration_sessions`
- 对应实体：`src/infrastructure/persistence/acceleration_session.rs`

字段（核心）：

- `session_id`（PK）
- `user_id`
- `game_id`
- `node_id`
- `admin_user_id`（重要：用于关联节点上报 `uid`；当前 start 时记录）
- `uuid`（下发到节点侧的 Xray 用户 UUID）
- `outbound_tag`
- `status`：`active | stopped | ...`
- `bill_type`：`pass | minute`
- `started_at`
- `last_activity_at`
- `last_accounted_at`
- `billed_minutes`
- `ended_at`

> 该表是对账/风控/争议处理的事实来源。

### 3.4 CDK（已有）

- 表：`cdk_codes`
- 对应实体：`src/infrastructure/persistence/cdk_code.rs`

字段：

- `code`
- `cdk_type`：`Day | Month | Year | Minute`
- `duration_minutes`：统一用“分钟数”存储（例如 Day=1440，Month=43200，Year=525600）
- `status`：`unused | used`
- `used_by` / `used_at`

---

## 4. 计费逻辑（实现状态说明）

### 4.1 CDK 兑换逻辑（已实现）

实现位置：

- `src/application/cdk_usecase.rs`

规则：

- `Day/Month/Year`：
  - 兑换后更新 `valid_until`：
    - 未过期：`valid_until += duration_minutes`
    - 已过期：`valid_until = now + duration_minutes`

- `Minute`：
  - 兑换后更新分钟钱包：
    - `user_wallets.remaining_minutes += duration_minutes`
  - **不会**把分钟兑换成 `valid_until`（避免变成“有效期”而不是“用量扣费”）

### 4.2 账号校验（已实现）

接口：`POST /api/account/validate`

返回：

- `is_valid`
- `is_paid`
- `valid_until`
- `remaining_minutes`
- `billing_mode`（`pass|minute|none`）

### 4.3 会话开始/结束（已实现）

接口：

- `POST /api/accelerator/session/start`
- `POST /api/accelerator/session/stop`

开始时会：

- 根据 `billing_mode` 决定 `bill_type`
- 创建 `admin_user/admin_user_mapping`（写 DB）
- 插入 `acceleration_sessions(status=active)`

结束时会：

- 删除 `admin_user`（内部也会清理 mapping）
- 将 `acceleration_sessions` 标记为 `stopped`

### 4.4 分钟扣费（服务端扣费）

**当前仓库：分钟余额的“扣减任务”尚未实现（规划中）。**

规划的扣费方式（推荐）：

- 由 `main.rs` 后台定时任务扫描 `acceleration_sessions` 中 `status=active AND bill_type=minute`
- 使用**服务器时间**结合节点上报（例如 `node_online_user_log` / `node_traffic_log`）判断会话是否仍在使用
- 每满 60 秒扣 1 分钟：
  - `user_wallets.remaining_minutes -= minutes_to_charge`
  - `acceleration_sessions.billed_minutes += minutes_to_charge`
  - 若余额不足：
    - 标记会话为 `insufficient_balance`
    - 并撤销节点侧用户（删除 `admin_user/admin_user_mapping`，同时通过 MQTT 通知节点重载）

---

## 5. Web API（client_api）接口清单

所有接口都在 `/api` scope 下（见 `src/interface/web/routes.rs`）。

### 5.1 CDK 相关

#### 5.1.1 生成 CDK（通常用于后台/运营）

- `POST /api/cdk/generate`

请求示例：

```json
{
  "cdkType": "Month",
  "count": 10
}
```

> 说明：这相当于“发卡/出货”。如果你要接入支付，“购买”通常是：支付成功 → 服务端生成/分配一个 CDK → 下发给客户端。

#### 5.1.2 兑换 CDK（客户端使用）

- `POST /api/cdk/redeem`

请求：

```json
{
  "code": "XXXX-XXXX",
  "userId": "user_123"
}
```

响应（示例）：

- 时长卡：

```json
{
  "success": true,
  "message": "...",
  "durationMinutes": 43200,
  "validUntil": "2026-01-30 12:00:00",
  "remainingMinutes": null
}
```

- 分钟包：

```json
{
  "success": true,
  "message": "...",
  "durationMinutes": 600,
  "validUntil": null,
  "remainingMinutes": 1200
}
```

#### 5.1.3 CDK 列表（后台用途）

- `GET /api/cdk/list?status=unused|used`

### 5.2 账号校验

#### 5.2.1 校验账号（客户端开始会话前建议调用）

- `POST /api/account/validate`

请求：

```json
{
  "userId": "user_123"
}
```

响应（示例）：

```json
{
  "isValid": true,
  "isPaid": false,
  "validUntil": null,
  "remainingMinutes": 120,
  "billingMode": "minute",
  "message": "Account has remaining minutes: 120"
}
```

### 5.3 会话（开始/停止加速）

#### 5.3.1 开始会话

- `POST /api/accelerator/session/start`

请求：

```json
{
  "userId": "user_123",
  "gameId": "game_abc",
  "nodeId": 1,
  "outboundTag": "ss-out-1",
  "chainId": null,
  "chainBasePort": null
}
```

响应：

```json
{
  "sessionId": "...",
  "uuid": "...",
  "billType": "pass",
  "remainingMinutes": null
}
```

说明：

- `uuid` 是本次会话下发到节点侧的用户标识（Xray/V2Ray 用户 UUID）。
- `billType` 对应 `billing_mode`：
  - `pass`：会员放行
  - `minute`：分钟计费
- `outboundTag` 用于决定节点侧将该用户流量导向哪个上游（与 `admin_user_mapping` 对应）。
- `chainId/chainBasePort` 为可选链路参数（多跳链路）；若传入，服务端会写入链路相关 inbounds。

#### 5.3.2 停止会话

- `POST /api/accelerator/session/stop`

请求：

```json
{
  "sessionId": "..."
}
```

响应：

```json
{
  "status": "stopped",
  "billedMinutes": 0
}
```

> 说明：`billedMinutes` 的递增需要配合“服务端后台扣费任务”（见 4.4）。

---

## 6. 客户端接入流程（推荐）

### 6.1 购买/发卡（年/月/日/分钟）

当前仓库的“购买”不是一个完整支付系统，而是通过 CDK 作为兑换媒介：

- 运营后台/支付回调服务：调用 `POST /api/cdk/generate` 生成 CDK
- 将 CDK 通过你自己的渠道发给用户（或支付完成后直接返回）

### 6.2 客户端兑换

用户输入 CDK 后：

1. 调用 `POST /api/cdk/redeem`
2. 成功后：
   - 若是日/月/年：客户端可展示 `validUntil`
   - 若是分钟：客户端可展示 `remainingMinutes`

### 6.3 客户端开始加速（开始会话）

1. （可选但推荐）先调用 `POST /api/account/validate`，获取 `billingMode` 和余额/到期时间
2. 用户选择：
   - 游戏（`gameId`）
   - 节点（`nodeId`）
   - 出站（`outboundTag`）
3. 调用 `POST /api/accelerator/session/start`
4. 得到 `sessionId/uuid/billType`：
   - `uuid` 用于节点侧识别该用户（后续也可用于排障/追踪）

### 6.4 客户端结束加速（停止会话）

1. 调用 `POST /api/accelerator/session/stop`
2. 服务端会撤销节点侧用户/映射，并结束会话记录。

---

## 7. 常见问题（FAQ）

### 7.1 为什么分钟计费不依赖客户端心跳？

- 客户端心跳容易伪造/篡改，会导致绕过计费。
- 推荐由服务端基于节点上报数据进行扣费，并以服务器时间为准。

### 7.2 为什么 start/stop 可以在 client_api 中直接写 DB？

- `client_api` 与 `main.rs` 分开部署但共享 DB，在同一内网。
- 节点侧配置下发的数据源本身就是 DB（`admin_user/admin_user_mapping/...`）。

### 7.3 分钟扣费什么时候会真正扣？

- 需要在 `main.rs` 实现后台扣费任务（见 4.4）。
- 当前代码已具备：分钟充值（wallet 增加）、会话记录、节点侧下发数据准备。

---

## 8. 相关代码位置索引

- Web 入口：`src/bin/client_api.rs`
- Web 路由：`src/interface/web/routes.rs`
- Web handlers：`src/interface/web/handlers.rs`
- DTO：`src/interface/web/dto.rs`
- CDK 用例：`src/application/cdk_usecase.rs`
- Admin 配置存储（写 DB）：`src/infrastructure/admin_config.rs`
- 会话表实体：`src/infrastructure/persistence/acceleration_session.rs`

---

## 9. 验证步骤（手工 / 最短闭环）

本节用于在本地/测试环境手工验证：

- 分钟钱包充值（CDK minute）
- 会话 start/stop 正常
- `main.rs` 后台分钟计费 daemon 会扣减 `user_wallets.remaining_minutes`
- 离线 / 余额不足时自动停止会话并撤销节点侧用户（best effort）

### 9.1 环境变量（建议）

两进程必须共享同一个数据库：

- `DATABASE_URL`：例如 `mysql://user:pass@127.0.0.1:3306/ntex_demo`

可选（为了更容易观察扣费/离线 stop）：

- `BILLING_TICK_SECONDS=5`
- `BILLING_ONLINE_GRACE_SECONDS=30`

client_api 端口：

- `PORT=8080`（默认）

### 9.2 启动进程

启动顺序建议：先 `main.rs`，再 `client_api`。

1. 启动管理端（含 MQTT broker + MQTT client + billing daemon）：

```bash
cargo run
```

2. 启动 Web API（client_api）：

```bash
cargo run --bin client_api
```

### 9.3 生成测试数据：节点 + minute CDK + 兑换

以下 curl 示例默认 client_api 在 `http://127.0.0.1:8080`。

#### 9.3.1 注册一个节点

`POST /api/nodes/register`

```bash
curl -sS -X POST "http://127.0.0.1:8080/api/nodes/register" \
  -H "Content-Type: application/json" \
  -d '{
    "id":"1",
    "vmessUuid":"00000000-0000-0000-0000-000000000001",
    "vmessServer":"example.com",
    "vmessPort":443,
    "vmessEmail":"u1@example.com",
    "udpProxy":"off",
    "mode":"vmess",
    "ping":10,
    "status":"online"
  }'
```

#### 9.3.2 生成一张 minute CDK（例如 5 分钟）

`POST /api/cdk/generate`

```bash
curl -sS -X POST "http://127.0.0.1:8080/api/cdk/generate" \
  -H "Content-Type: application/json" \
  -d '{"cdkType":"minute","count":1,"durationMinutes":5}'
```

返回结果里会包含 `code`，把它复制出来备用。

#### 9.3.3 兑换 CDK（例：userId = u1）

`POST /api/cdk/redeem`

```bash
curl -sS -X POST "http://127.0.0.1:8080/api/cdk/redeem" \
  -H "Content-Type: application/json" \
  -d '{"code":"<PUT_CODE_HERE>","userId":"u1"}'
```

兑换成功后，minute 类型会返回 `remainingMinutes`。

#### 9.3.4 校验账号（确认 billingMode/余额）

`POST /api/account/validate`

```bash
curl -sS -X POST "http://127.0.0.1:8080/api/account/validate" \
  -H "Content-Type: application/json" \
  -d '{"userId":"u1"}'
```

预期：

- `billingMode` 为 `minute`
- `remainingMinutes > 0`

### 9.4 开始会话（创建 active session + 下发节点用户/映射）

`POST /api/accelerator/session/start`

```bash
curl -sS -X POST "http://127.0.0.1:8080/api/accelerator/session/start" \
  -H "Content-Type: application/json" \
  -d '{
    "userId":"u1",
    "gameId":"g1",
    "nodeId":1,
    "outboundTag":"default"
  }'
```

预期返回：

- `sessionId`
- `uuid`（下发到节点侧的用户 UUID）
- `billType`（`minute` 或 `pass`）
- minute 模式会带 `remainingMinutes`

### 9.5 让 billing daemon 认为“用户在线”（关键）

分钟计费 daemon 不依赖客户端心跳，它只看 `node_online_user_logs`。

`node_online_user_logs` 由节点通过 MQTT 上报 onlineusers 写入（`main.rs` 的 MQTT 通路会调用 `AdminConfigStore::handle_online_users_report`）。

因此验证在线最简方式：

1. 让节点（或你的 node 端）持续上报 onlineusers
2. 确认数据库表 `node_online_user_logs` 中，存在：
   - `node_id = session.node_id`
   - `user_id = session.admin_user_id`
   - `created_at` 在 `BILLING_ONLINE_GRACE_SECONDS` 时间窗口内持续刷新

> 说明：这里的 `user_id` 指的是 `acceleration_sessions.admin_user_id`（节点上报里的 uid），不是业务侧的 `acceleration_sessions.user_id`（例如 u1）。

### 9.6 观察扣费

保持用户“在线”一段时间（至少跨过 1 分钟的计费窗口）后：

- `user_wallets.remaining_minutes` 会下降
- `acceleration_sessions.billed_minutes` 会上升
- `acceleration_sessions.last_accounted_at` 会推进

同时 `main.rs` 日志会打印类似：

- `session billed: ... minutes_charged=... remaining_minutes=...`

### 9.7 触发自动停机：余额不足 / 离线

#### 9.7.1 余额不足

minute 余额不足时，daemon 会将会话置为 `insufficient_balance` 并撤销节点侧用户（best effort）：

- `acceleration_sessions.status = insufficient_balance`
- `acceleration_sessions.ended_at` 写入
- `AdminConfigStore::delete_user(node_id, admin_user_id)` 被调用
- MQTT 更新通知 best effort 发送到：`user/outbound/inbound/config`

#### 9.7.2 离线

如果 `node_online_user_logs` 在 `BILLING_ONLINE_GRACE_SECONDS` 内不再刷新，则认为用户离线并自动 stop：

- `acceleration_sessions.status = stopped`
- `acceleration_sessions.ended_at` 写入
- 同样会撤销节点侧用户并发送 MQTT 更新（best effort）

- 钱包表实体：`src/infrastructure/persistence/user_wallet.rs`
- 设计文档：`BILLING_PLAN.md`
