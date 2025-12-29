# 游戏加速器计费方案（设计文档）

## 1. 目标与范围

本文档描述在现有仓库（`ntex-demo` + `XrayR`）基础上，实现以下业务效果所需的计费与会话设计：

- 客户端选择游戏与节点后点击“开始加速”。
- 服务端生成一个与用户绑定的 `uuid`（作为 Xray/V2Ray 用户 ID），并下发到对应节点。
- 客户端通过国内网关的透明代理端口转发，走固定链路（可选 `chain_ops` 多跳），访问最终节点。
- 支持并可扩展以下计费形态：
  - **日卡**
  - **月卡**
  - **年卡**
  - **分钟计费**（按会话使用时长扣减）

本文档以“可落地、最小改造、可对账”为设计原则。

## 2. 现有实现概览（代码对应）

### 2.1 网关/客户端连接服务（Web）

- 入口：`src/bin/client_api.rs`
  - 启动 `interface::web::serve(port, state)`
- Web 路由：`src/interface/web/routes.rs`
- Web 处理器：`src/interface/web/handlers.rs`
  - 当前 `POST /api/accelerator/start` 仅进行 `valid_until` 校验（CDK/账号验证），**尚未实现**：
    - 生成 `uuid`
    - 建立“加速会话”
    - 将 `uuid` 下发到节点

### 2.2 管理服务（Admin）与 MQTT 通路

- 入口：`src/main.rs`
  - 启动 MQTT Broker + MQTT Client
  - 启动管理服务器：`admin::serve(667, ...)`
- Admin 路由：`src/interface/admin/routes.rs`
  - 存在用户管理：`add_user/update_user/delete_user`
  - 存在用户映射管理：`add_mapping/update_mapping/delete_mapping`
  - 存在链路配置：`apply_chain`（多跳端口转发能力）
- 管理配置存储：`src/infrastructure/admin_config.rs`
  - `add_user(node_id, uuid, st, dt)` 写入 `admin_user`
  - `admin_user_mapping`（存在映射概念，供“uuid -> outbound tag/链路 tag”）

### 2.3 节点侧（XrayR）

- 代码在 `XrayR/`，并包含 `api/mqttpanel`：
  - 节点可通过 MQTT 拉取 `user/config/outbound` 等，并接收“更新通知”。

结论：

- “下发用户到节点”的数据面已经具备（Admin DB + MQTT 通知 + XrayR 拉取）。
- 目前缺少的是“面向客户端的会话/计费 API”，以及“分钟计费钱包/会话表”。

## 3. 计费产品定义

### 3.1 计费形态

- **时长型套餐**：日卡 / 月卡 / 年卡
  - 购买后得到 `valid_until`（到期时间），未到期可无限次开始会话。
- **用量型套餐**：分钟计费
  - 购买后得到 `remaining_minutes`（剩余分钟数）。
  - 只有在会话 `active` 期间按分钟扣减。

### 3.2 套餐优先级（强烈建议）

当用户发起开始加速：

1. 如果 `now < valid_until`：
   - **按会员放行**，不消耗分钟余额。
2. 否则如果 `remaining_minutes > 0`：
   - **允许开始**，并对该会话按分钟扣费。
3. 否则拒绝开始。

理由：

- 兼容你现有 `valid_until` 逻辑。
- 分钟计费可以作为兜底，降低用户门槛。
- 对账更清晰：会员与分钟计费互不混淆。

## 4. 核心对象与数据模型

### 4.1 用户与会员到期（现有）

- 现有：`accelerator_users.valid_until`
  - 已用于 `validate_account()` 判断。

### 4.2 新增：分钟钱包（推荐新增表）

建议新增 `user_wallet`（或在用户表加字段作为第一阶段临时方案）：

- `user_id` (PK/Unique)
- `remaining_minutes` (int)
- `updated_at`

说明：

- `remaining_minutes` 仅对分钟计费生效。
- 会员未到期时不扣减该值。

### 4.3 新增：加速会话（计费/风控/对账核心）

建议新增 `acceleration_sessions`：

- `id`：`session_id`（UUID v4 或 ULID）
- `user_id`
- `game_id`
- `node_id`（用户选择的节点）
- `uuid`（下发到节点、并给客户端使用的 Xray 用户 UUID）
- `status`：
  - `active`
  - `stopped`
  - `expired`（会员到期导致会话停止，仅用于对账标记）
  - `insufficient_balance`（余额不足导致停止）
- `bill_type`：
  - `pass`（会员/时长型）
  - `minute`（分钟计费）
- `started_at`
- `last_heartbeat_at`
- `ended_at`
- `billed_minutes`（累计扣除分钟数，仅 `bill_type=minute` 时递增）
- `price_snapshot`（可选：用于未来引入阶梯/不同游戏不同单价）

关键点：

- 该表是所有计费纠纷的“唯一事实来源”。
- 后续如要加“按流量计费/按游戏计费”，可以在该表追加字段或增加账单明细表。

### 4.4 节点侧用户限制字段（复用 admin_user）

`admin_user` 已存在：

- `uuid`
- `st`：限速（Mbps）
- `dt`：设备限制

建议将“套餐档位”映射到 `st/dt`：

- 日卡：`st=50, dt=1`（示例）
- 月卡：`st=100, dt=2`
- 年卡：`st=200, dt=3`
- 分钟计费：与月卡一致或较低（防止低门槛用户资源占用过大）

具体数值按运营策略配置化（可放到 `config_entries`）。

## 5. 会话生命周期与状态机

### 5.1 状态机

- `active`：允许流量通过（用户已在节点存在、映射存在、链路可用）
- `stopped`：用户停止或超时停止
- `expired`：会员到期触发停止
- `insufficient_balance`：分钟余额不足触发停止

### 5.2 生命周期（核心流程）

1. **Start（开始加速）**
   - 鉴权用户（token 或 user_id）
   - 校验资费：
     - `valid_until` 或 `remaining_minutes`
   - 生成 `uuid`（与用户绑定）：
     - 推荐每次会话生成一个新 `uuid`（避免复用导致的并发/作弊难处理）
   - 写入 `acceleration_sessions` 为 `active`
   - 下发到节点：
     - `admin_user` 增加该 `uuid`
     - `admin_user_mapping` 建立该 `uuid` 到 outbound/链路 tag 的映射
     - （可选）触发 `apply_chain` 确保链路端口存在

2. **Heartbeat（心跳）**
   - 客户端定期上报（例如每 30s/60s）
   - 服务端更新 `last_heartbeat_at`
   - 若分钟计费：按规则扣减 `remaining_minutes` 并递增 `billed_minutes`
   - 若余额不足：
     - 将 session 置为 `insufficient_balance`，并触发撤销下发（删除 uuid、删除映射）

3. **Stop（停止）**
   - 客户端主动停止
   - 服务端进行最终结算（分钟计费则补齐最后一段）
   - `ended_at` 写入，状态变更为 `stopped`
   - 撤销节点侧 `uuid`（删除用户/映射）

4. **Timeout Stop（超时停止）**
   - 如果客户端异常退出不发 stop
   - 后台任务扫描：`now - last_heartbeat_at > timeout`（例如 90s 或 2 * 心跳间隔）
   - 自动 stop 并结算

## 6. 分钟计费规则（推荐实现）

### 6.1 计费粒度

- 心跳间隔：60 秒（推荐）
- 计费粒度：按分钟扣减（每满 60 秒扣 1 分钟）

### 6.2 计费算法（伪代码）

- 在 `heartbeat(session_id)`：

1. 读取 session，确认 `status=active`
2. `delta = now - last_heartbeat_at`
3. `minutes_to_charge = floor(delta / 60s)`
4. 如果 `minutes_to_charge > 0`：
   - 在同一事务中：
     - `user_wallet.remaining_minutes -= minutes_to_charge`（不得减成负数）
     - `session.billed_minutes += minutes_to_charge`
     - `session.last_heartbeat_at += minutes_to_charge * 60s`（而不是直接等于 now，避免重复扣）
5. 如果余额不足：
   - `session.status = insufficient_balance`，并触发撤销下发。

### 6.3 最小余额要求

- `start` 时要求至少 `remaining_minutes >= 1`
  - 避免“开始即余额不足”的边界纠纷。

## 7. 时长型套餐（日/月/年）规则

- 使用现有 `valid_until`：
  - `validate_account` 逻辑保持不变：`user.valid_until > now`
- 如果要支持“续费叠加”：
  - 若用户尚未过期：`valid_until += duration`
  - 若已过期：`valid_until = now + duration`

## 8. 与 CDK 系统的对齐

### 8.1 现有 CDK 类型

- `src/domain/cdk.rs`：`CdkType::{Day, Month, Year, Minute}`

### 8.2 建议的兑换处理

- `Day/Month/Year`：
  - 兑换后延长 `user.valid_until`
- `Minute`：
  - 兑换后增加 `user_wallet.remaining_minutes += duration_minutes`
  - 不建议把 minute 兑换成 `valid_until += N minutes`，否则会变成“有效期”而不是“按使用扣费”。

### 8.3 账号验证（validate_account）建议升级

`validate_account` 返回 `is_valid` 的条件建议改为：

- `now < valid_until` **或** `remaining_minutes > 0`

并在响应中返回：

- `valid_until`
- `remaining_minutes`
- `billing_mode`（pass/minute/none）

## 9. 面向客户端的 API 设计（Web：`handlers.rs`）

建议将 `POST /api/accelerator/start` 升级为“会话 API”，并新增心跳/停止。

### 9.1 开始会话

- `POST /api/accelerator/session/start`

请求：

- `user_id`
- `game_id`（或 profile_id）
- `node_id`（用户选定）
- `chain_id`（可选：固定链路 id）

响应：

- `session_id`
- `uuid`
- `gateway_host`
- `gateway_port`（透明代理入口端口）
- `heartbeat_interval_sec`
- `bill_type`：`pass` / `minute`
- `valid_until`（如果是 pass）
- `remaining_minutes`（如果是 minute）

### 9.2 心跳

- `POST /api/accelerator/session/heartbeat`

请求：

- `session_id`

响应：

- `status`
- `remaining_minutes`（可选）

### 9.3 停止

- `POST /api/accelerator/session/stop`

请求：

- `session_id`

响应：

- `status`
- `billed_minutes`

## 10. Web 如何下发到节点（推荐路径）

由于 Web（`client_api.rs`）当前没有 MQTT client，且 XrayR 下发是通过 Admin/MQTT 通路完成，建议：

- Web 在 `start_session` 中调用 Admin 的 HTTP API（本机或内网）：
  - `POST /api/admin/user?node_id=...`：创建 `admin_user`
  - `POST /api/admin/mapping?node_id=...`：创建 `uuid -> outbound_tag/chain_tag` 映射
  - （可选）`POST /api/admin/chains/apply`：确保链路端口已建立

Admin 服务端将：

- 写 DB（`admin_user/admin_user_mapping`）
- 通过 MQTT 发布更新通知，促使节点 XrayR 重载配置

## 11. 固定链路（透明代理端口）与计费的关系

### 11.1 链路入口

- 建议对客户端暴露一个“国内网关入口端口”，客户端只连这个端口。
- 该端口可以对应：
  - 固定第一跳节点（CN Gateway Node）
  - 或由 `chain_id` 决定第一跳

### 11.2 链路配置

- 你已有 `chain_ops::apply_chain` 能自动在多节点创建 `dokodemo-door` 入站，实现端口级链路转发。
- 计费不依赖链路实现，计费基于“会话是否 active”。

## 12. 风控与异常处理（强制建议）

### 12.1 并发会话

建议约束：同一用户同一时刻仅允许 1 个 `active` 会话。

- 再次 start：
  - 要么拒绝
  - 要么自动 stop 旧会话并启动新会话（更友好）

### 12.2 节点切换

- 节点切换本质是 stop + start。
- 必须撤销旧 uuid（删 user/mapping），避免“多节点同时可用”造成绕计费。

### 12.3 余额扣减一致性

- 分钟扣费必须事务化，避免并发心跳造成重复扣：
  - 方案 A：数据库事务 + 行级锁
  - 方案 B：乐观锁版本号

### 12.4 对账

- 纠纷时以 `acceleration_sessions` 为准：
  - `started_at`
  - `ended_at`
  - `billed_minutes`
  - `status`

## 13. 最小落地步骤（推荐里程碑）

1. **新增表/字段**：`user_wallet`、`acceleration_sessions`
2. **升级 CDK minute 兑换**：将 minute 写入 `remaining_minutes`
3. **实现会话 API**：start/heartbeat/stop + 超时任务
4. **实现下发调用链**：Web 调 Admin（add_user + mapping + apply_chain）
5. **风控与并发处理**：单用户单会话、节点切换策略

## 14. 需要产品侧确认的参数（实现前必须定）

- 分钟计费是“预付分钟包”还是“后付费扣人民币”？（本文档默认预付分钟包）
- `dt` 是否作为“并发设备限制”？
- 链路入口是固定网关节点还是随 `chain_id` 变化？
- 心跳间隔与超时时间：60s / 90s 是否可接受？

