# MQTT → gRPC 节点管理迁移方案（保持对外 API 行为不变）

本计划在不改变 `src/interface/admin/handlers.rs` 与 `src/interface/web/handlers.rs` 的对外 HTTP 行为（路由、请求/响应结构、状态码语义）前提下，将内部节点通信从 MQTT 逐步迁移到 gRPC（优先使用 443/TLS），并允许对数据库与后端架构进行大跨度调整。

## 0. 执行前置条件（开始改代码前）

- 允许对仓库源码进行变更：`src/**` 与 `XrayR/**`（后续会以“可回滚的最小批次”逐步提交）。
- 第一个实现批次（优先做，风险最低、可回滚）：为 `admin_node_configs` 增加 `node_token/node_shared_secret/node_comm_mode` 字段，并在 `database.rs` 增加对应迁移逻辑，确保 `seed` 与老数据兼容。

## 1. 现状关键结论（以当前代码为准）

### 1.1 后端（Rust）当前 MQTT 承担的职责

- **节点拉取配置（node -> server）**：节点通过 `xrayr/node/{node_id}/request/{request_id}` 请求 `action=user/config/inbound/outbound/routing`，服务端在 `src/infrastructure/mqtt_client.rs::handle_message/handle_request` 从 DB 取数据并回包。
- **更新通知（server -> node）**：服务端发布 `xrayr/node/{node_id}/update/{type}`，节点收到后触发刷新。
- **强一致 sync 语义（admin API）**：`/api/admin/user` 等接口支持 `sync=1`，服务端发送 update 后用 `wait_for_node_pull(node_id, action)` 等待观察到节点“实际拉取”。
- **管理指令（server -> node -> server）**：
  - `query_node_logs`（日志查询）
  - `query_node_network_interfaces`（网卡信息）
  - `udp_probe`（UDP 探测）
  都是服务端发 request，节点回 response。
- **节点上报（node -> server）**：`submit/nodestatus/onlineusers/illegal/outbound_*` 等上报更新 DB。
- **Presence 在线状态**：服务端收到消息更新内存 presence 并写 DB（`is_online/last_seen_at`），watchdog 周期性标记离线。

### 1.2 节点程序（XrayR）当前 MQTTPanel 的协议形态

`XrayR/api/mqttpanel/mqttpanel.go` 已实现：

- 订阅 `response/+`、`update/+`、`request/+`。
- 节点主动处理来自服务端的 request（`query_logs/udp_probe/query_network`）并回包。
- 节点收到 update 通知后触发 `updateCallback`，Controller 侧会“立即更新配置”。

结论：现有 MQTT 的功能可以 1:1 映射到 gRPC（Unary + Streaming/Bidi）。

## 2. 迁移总体设计（目标状态）

### 2.1 对外不变的边界

- **不改变**：
  - Admin API：`/api/admin/...`（含 `sync`、`sync_timeout`、错误码/错误信息语义）
  - Client/Web API：`/api/...`（例如 session_start 对节点在线的判断、返回 Profile 结构等）
- **允许改变**：
  - 后端内部模块划分、服务部署方式（单体/拆分）、数据库表结构与存储模型
  - 节点与服务端的内部通信协议（MQTT → gRPC）

### 2.2 新的内部抽象：NodeTransport（消除“MQTT 直连到业务代码”）

在 Rust 后端引入一个稳定抽象层（名称可调整）：

- `trait NodeTransport`（或 `NodeGateway`）提供：
  - `publish_update(node_id, kind)`
  - `wait_for_node_pull(node_id, action, timeout)`
  - `query_logs(node_id, params, timeout)`
  - `query_network(node_id, timeout)`
  - `udp_probe(node_id, outbound_tag, timeout)`
  - `report_*`（如果上报仍由 transport 接收/转发）

实现两套：

- **`MqttTransport`**：现有逻辑的封装（复用 `MqttClientManager/MqttPublisher`）
- **`GrpcTransport`**：新 gRPC 通道（推荐）

并在 `AdminState/AppState` 中注入该抽象，确保 `handlers` 对“MQTT vs gRPC”无感。

### 2.3 gRPC 连接模型（跨境更稳）

- **连接方向**：节点主动出站连接到服务端（更符合跨境网络/防火墙/NAT 场景）。
- **通道类型**：`Bidi Streaming`（一条长连接承载 push + request/response + 上报）。
- **入口端口**：最终统一到 **443/TLS**（在程序内启动 443 入口，提供证书维护目录；推荐用子域 + SNI 分流承载 HTTP 与 gRPC）。

### 2.4 gRPC 协议映射（与现有 MQTT 语义等价）

- **更新通知**：server 在 stream 下发 `UpdateEvent{kind, version, ts}`。
- **配置拉取**：node 收到 `UpdateEvent` 后调用 unary RPC `GetUsers/GetOutbounds/...`（或在 stream 里请求），并在完成后回传 `PullAck{kind, version}`。
  - `wait_for_node_pull()` 改为等待 `PullAck`，语义等价于当前“观察到 node pull request”。
- **指令请求（query_logs/query_network/udp_probe）**：server 下发 `CommandRequest{request_id, kind, payload}`，node 回 `CommandResponse{request_id, ok, payload}`。
- **节点上报**：node 定期发送 `Report{kind, payload}`（traffic/status/online_users/illegal/outbound_events）。
- **Presence**：服务端以 stream 心跳或任意消息更新时间，watchdog 标记离线。

## 3. 迁移阶段（可灰度/可回滚）

### 阶段 0：准备工作（不影响生产）

- **抽象改造**：引入 `NodeTransport` 接口，先用 `MqttTransport` 适配现状。
- **协议定义**：新增 `proto/node_control.proto`（仅内部使用），定义：
  - `NodeControlService/NodeStream`（bidi stream）
  - `GetUsers/GetConfig/GetInbounds/GetOutbounds/GetRouting`（unary）
  - `CommandRequest/CommandResponse/UpdateEvent/PullAck/Report/Heartbeat`
- **gRPC 服务端骨架**：Rust 用 `tonic` 实现基础服务，但先不切流量。

验收：功能不变；单元/集成测试能跑；不需要节点升级。

### 阶段 1：后端双栈（MQTT + gRPC）

- 生产仍用 MQTT；同时在后端上线 gRPC server：
  - 维护 `connected_nodes` registry（node_id → stream sender + last_seen + capabilities）。
  - 将 `GrpcTransport` 做成可用（即使暂时没有节点连接）。
- 引入 **每节点 transport 选择**（策略）：
  - 通过 DB 字段或配置：`node_comm_mode = mqtt|grpc|auto`。
  - `auto` 优先 gRPC（节点连上则走 gRPC，否则 fallback MQTT）。

验收：后端部署与运行稳定；对外 API 行为未变；MQTT 仍是主路径。

### 阶段 2：节点侧新增 gRPCPanel（或 dual-stack）

- 在 XrayR 新增 `api/grpcpanel`（推荐新写）：
  - 连接 gRPC server，开启 stream
  - 实现：更新通知、配置拉取、日志查询、udp_probe、网络接口查询、上报
  - 保持与 Controller 的交互方式不变（继续使用 `SetUpdateCallback` 触发立即刷新）
- 配置开关：
  - 新增 PanelType：`GRPC`（或保持 PanelType=MQTT 但内部可切换）
  - `ApiHost` 由 broker 地址变为 gRPC 地址（建议 `https://host:443`）

验收：单节点在跨境网络下稳定在线；admin 的 `sync=1` 可正常等待/回滚；query_logs/udp_probe 正常。

### 阶段 3：灰度切换（按 node_id）

- 先挑选跨境问题最严重的节点切到 gRPC。
- 指标与回滚：
  - 连接在线率、重连次数、命令超时率、sync 超时率
  - 可一键切回 `node_comm_mode=mqtt`

验收：跨境节点稳定性显著改善；业务无感。

### 阶段 4：下线 MQTT（可选最终目标）

- 关闭 `mqtt_broker` 内嵌 broker（`src/infrastructure/mqtt_broker.rs`）
- 清理 `MqttClientManager/MqttPublisher` 依赖与相关环境变量
- 对公网只保留 443（通过网关统一入口）

## 4. 数据库与状态模型调整（允许大改，建议方向）

- **节点连接与能力表**（建议新增）：
  - `node_connection(node_id, mode, last_seen_at, is_online, endpoint, version, capabilities)`
- **事件/同步状态**（用于替代 MQTT pull waiter 的“观察拉取”）：
  - `node_sync_state(node_id, kind, last_ack_version, last_ack_at)`
  - 或仅内存 + DB last_seen_at（依赖需求强一致程度）
- **命令审计（可选）**：
  - `node_command_log(request_id, node_id, kind, status, latency_ms, created_at)`

已确认落地选择（便于快速上线，减少新表引入）：

- **节点鉴权与通信模式字段**：直接扩展 `admin_node_configs`：
  - `node_token`（节点 token，用于快速定位与轮换）
  - `node_shared_secret`（每节点独立 shared_secret，用于签名校验）
  - `node_comm_mode`（`mqtt|grpc|auto`）
  - 可选：`node_secret_rotated_at`（便于审计与强制轮换）
- **索引建议**：
  - `admin_node_configs(node_id)` 已为主键
  - 如 `node_token` 允许外部检索，则对 `node_token` 建唯一索引（避免重复配置）

## 5. 安全与端口策略（跨境 + 端口安全）

- **入口统一 443**：程序内启动 TLS 入口（提供证书维护目录，例如 `TLS_CERT_DIR`），推荐通过 **子域 + SNI** 分流到后端 HTTP 与 gRPC，避免同 host 下 HTTP/2 与 gRPC 复用带来的复杂度。
- **证书**：使用 wildcard 证书（由运维侧配置与维护证书文件）。
- **鉴权**：token + 签名（建议 HMAC-SHA256；每节点独立 `shared_secret`；签名信息放 gRPC metadata；包含 `node_id/timestamp/nonce/body_hash`，并做重放保护）。
- **限流/配额**：按 `node_id` / token 限制连接数、命令频率、消息大小。
- **大响应处理（日志）**：内部采用分页/分块（server-streaming 或分页 RPC）；对外 admin 接口保持现有 JSON 响应结构不变（服务端聚合后返回）。

## 6. 关键兼容点（必须保持）

- **`sync=1` 语义**：现在是“服务端发 update → 等待节点实际拉取”。迁移后必须用 `PullAck(kind, version)` 或等价机制保证。
- **`NODE_OFFLINE` 判断**：web `session_start` 依赖 DB 的 `is_online/last_seen_at`。gRPC 必须稳定更新这两个字段。
- **admin 指令接口**：`query_logs/query_network/udp_probe` 必须有与现有 JSON 响应等价的数据结构。

## 7. 已确认的实现约束（落地选择）

- **入口形态**：无统一网关；在程序内启动 443/TLS 入口，提供证书维护目录；推荐子域 + SNI 分流（同一 443 端口）。
- **证书**：你已具备 wildcard 证书（你负责配置）。
- **鉴权**：token + 签名；每节点独立 shared_secret。
- **日志查询**：内部分页/分块；对外保持现有接口响应不变。
- **升级策略**：允许按节点灰度；允许双栈运行一段时间。

## 8. 执行拆解（实现阶段，里程碑交付）

### 里程碑 A：协议与服务端骨架（不切流量）

- 新增 `proto/node_control.proto` 与 Rust `tonic` 服务端骨架（stream + unary + command + report + ack）。
- 完成 token+签名鉴权中间件（仅用于 node gRPC；不影响现有 web Bearer token 体系）。
- 新增连接 registry：`node_id -> stream sender + last_seen + capabilities`；接入离线判定（更新 DB `is_online/last_seen_at`）。

落地位置建议：

- **gRPC server 归属进程**：优先挂在 `src/main.rs`（admin 进程）中启动，因为它已经持有 `AdminConfigStore + db + mqtt_client`，便于复用数据读取与在线状态落库。
- **gRPC 内部监听端口（示例）**：`127.0.0.1:50051`（仅供本机 443 入口转发）。
- **依赖新增（实现时）**：`tonic`/`prost`/`prost-types`/`tokio-stream`/`tower`（用于 interceptor 与 stream 管理）。

运行拓扑建议（不强制合并现有进程，改动最小）：

- `ntex-demo`（admin + mqtt）继续监听 `667`，并额外监听 `127.0.0.1:50051`（gRPC）。
- `client_api`（web）继续监听 `PORT`（默认 8080）。
- 新增一个轻量 `edge_gateway`（新 bin）监听 `443/TLS`，按 SNI 转发到 `667/8080/50051`。

presence 写入策略（减少 DB 写放大）：

- 复用 `mqtt_client.rs` 的思路：
  - 内存记录 `last_seen` 与 `is_online`。
  - **在线切换**（false->true / true->false）才写 DB。
  - `last_seen_at` 可按固定间隔（例如 30s）批量刷新，避免每个消息都 update。

验收：服务端能启动；节点未升级时不影响现网；对外 admin/web API 行为无变化。

### 里程碑 B：NodeTransport 抽象与 MQTT 适配（后端可双栈）

- 在后端引入 `NodeTransport`/`NodeGateway` 抽象。
- 实现 `MqttTransport`（复用当前 `MqttClientManager/MqttPublisher`）。
- 引入 per-node 策略开关：`node_comm_mode=mqtt|grpc|auto`（auto 优先 gRPC，未连接则回退 MQTT）。

验收：全量节点仍走 MQTT；`sync=1`、`query_logs/udp_probe/query_network` 现有行为不变。

### 里程碑 C：程序内 443/TLS 入口（证书目录 + SNI 分流）

- 在同一进程中新增 `443` 监听（`tokio-rustls`）。
- 证书目录约定（示例）：
  - `TLS_CERT_DIR=/path/to/certs`
  - `TLS_CERT_DIR/fullchain.pem`
  - `TLS_CERT_DIR/privkey.pem`
- **SNI 分流建议**（同一 443 端口）：
  - `grpc.<domain>` -> 转发到本地 gRPC（例如 `127.0.0.1:50051`）
  - `api.<domain>` / `admin.<domain>` -> 转发到现有 HTTP（例如 `127.0.0.1:667` / `client_api` 端口）

鉴权细节（建议落地约定）：

- 节点侧携带：
  - `x-node-id`
  - `x-node-token`
  - `x-ts`（秒级时间戳）
  - `x-nonce`（随机串）
  - `x-sign`（HMAC-SHA256(base_string)）
- base_string（stream / unary 通用建议）：`node_id\nnode_token\nts\nnonce`
- 可选增强（unary 或应用层自校验）：追加 `\nbody_sha256_hex`（stream 场景 interceptor 无法稳定拿到每帧 body，建议只做连接建立时鉴权，后续依赖 TLS）。
- 服务端校验：
  - `ts` 允许窗口（例如 ±300s）
  - `nonce` 在窗口内去重（内存或 DB）
  - `shared_secret` 从 DB 按 `node_id` 获取（每节点独立）

DB 字段建议（为兼容现有 seed/启动流程）：

- `node_token`：可为空（NULL），便于先上线后补配置。
- `node_shared_secret`：可为空（NULL），便于先上线后补配置。
- `node_comm_mode`：非空，默认 `mqtt`（确保老节点/未配置节点不受影响）。

验收：不改变现有端口的同时，新增 443 可用；证书可通过替换目录文件完成轮换（重启或热加载按实现选择）。

### 里程碑 D：XrayR gRPCPanel（双栈灰度）

- 新增 `api/grpcpanel`：连接 `grpc.<domain>:443`，实现 stream + 拉取 + ack + command + report。
- 日志查询支持分页/分块（内部）；后端聚合后保持 admin 旧响应结构。
- 灰度策略：按 node_id 分批启用 `node_comm_mode=grpc|auto`，失败回退 mqtt。

验收：跨境节点稳定在线；`sync=1` 不劣化；命令类接口成功率提升；可随时回滚 MQTT。

### 里程碑 E：下线 MQTT（可选）

- 当所有节点稳定迁移后，逐步关闭内嵌 broker 与 MQTT 相关依赖。

## 9. 代码落点地图（按当前代码勘察结果）

- **节点在线/离线状态写入（可复用）**：
  - `src/infrastructure/mqtt_client.rs`
    - `handle_message(...)`：收到任意 node topic 后更新内存 presence，首次在线时调用 `AdminConfigStore::set_node_online_status(node_id, true)`
    - `run_presence_watchdog(...)`：超时后调用 `AdminConfigStore::set_node_online_status(node_id, false)`
  - `src/infrastructure/admin_config.rs`
    - `set_node_online_status(node_id, is_online)`：更新 `admin_node_configs.is_online/last_seen_at`

- **admin 指令类接口当前入口（需要 transport 抽象接管）**：
  - `src/interface/admin/handlers.rs`
    - `refresh_node_network_interfaces` -> `mqtt_client.query_node_network_interfaces(...)`
    - `query_handler` 的 `act=user_logs` -> `mqtt_client.query_node_logs(...)`
    - 多处 `sync=1` -> `mqtt_client.publish_update_notification(...)` + `mqtt_client.wait_for_node_pull(...)`

- **MQTT 侧 request/response 的现有实现（后续 gRPC 需要对齐语义）**：
  - `src/infrastructure/mqtt_client.rs`
    - `wait_for_node_pull(node_id, action, timeout)`：当前“观察到 node pull request”的强一致信号
    - `query_node_logs(...)`：日志查询入口（后续内部分页/分块也从这里迁移到 gRPC）
    - `query_node_network_interfaces(...)`
    - `query_udp_latency(...)`

- **DB 初始化/迁移入口（新增字段会落在这里）**：
  - `src/infrastructure/database.rs`
    - `init(db)`：统一建表 + 迁移逻辑入口
    - `migrate_node_config_fields(db)`：当前已包含 admin_node_configs 多字段迁移（后续可在这里追加 node_token/node_shared_secret/node_comm_mode）


## 10. 当前已落地实现（以代码为准）

本章节用于描述“当前仓库已经实现到哪一步”，以及如何在本地/测试环境把链路跑起来并验证。

### 10.1 已实现功能清单

- **数据库字段**：`admin_node_configs` 已新增/迁移：
  - `node_token` / `node_shared_secret` / `node_comm_mode`
- **后端抽象**：`NodeTransport` + `MqttTransport` + `GrpcTransport` + `DispatchTransport` 已落地。
  - `DispatchTransport` 会根据 `admin_node_configs.node_comm_mode` 选择走 MQTT 或 gRPC。
- **gRPC Server 骨架**：
  - `proto/nodepanel.proto`：`Ping`（用于连通性验证）
  - `proto/node_control.proto`：`NodeStream`（bidi stream）+ `GetUsers/GetConfig/GetInbounds/GetOutbounds/GetRouting`（unary，占位）
- **gRPC 鉴权骨架**：metadata 中基于 `node_token + node_shared_secret` 的 HMAC 签名校验 + `nonce` 重放保护。
- **TLS 443 入口（SNI TCP 转发）**：
  - 支持 `grpc.<domain>` / `admin.<domain>` / `api.<domain>` 的 SNI 分流
  - 支持连接数限制、TLS 握手超时、后端连接超时、可选会话超时、ALPN 观测与更清晰日志

### 10.2 当前业务流程（server/admin 视角）

#### 10.2.1 Admin API 修改配置 -> 通知节点刷新

- Admin API（HTTP）修改 DB（例如 add inbound/outbound/user/routing）
- 业务代码调用 `NodeTransport::publish_update_notification(node_id, kind)`
  - `node_comm_mode=mqtt`：发布 MQTT topic `xrayr/node/{node_id}/update/{kind}`
  - `node_comm_mode=grpc`：在 gRPC stream 下发 `UpdateEvent{kind}`
  - `node_comm_mode=auto`：仅对灰度节点优先 gRPC，否则走 MQTT

#### 10.2.2 Admin API sync=1 强一致等待

- 当 admin 请求带 `sync=1` 时（例如 add_user/add_mapping/delete_user 等）
- 服务端在发出 update 后调用 `NodeTransport::wait_for_node_pull(node_id, kind, timeout)`
  - `mqtt`：等待观察到节点“实际 pull”（现有 MQTT 逻辑）
  - `grpc`：等待节点在 stream 上报 `PullAck{kind}`

说明：当前 gRPC 侧仅实现了 `PullAck` 的等待/计数机制；节点侧尚未实现真正的拉取与回传 ack（需要 XrayR gRPCPanel）。

### 10.3 gRPC 接口（proto）与使用方式

#### 10.3.1 NodePanelService（连通性/鉴权验证）

- **proto**：`proto/nodepanel.proto`
- **RPC**：`Ping(PingRequest) -> PingReply`

建议用于：

- 验证服务端已启动
- 验证 metadata 鉴权是否通过

#### 10.3.2 NodeControlService（节点主通道）

- **proto**：`proto/node_control.proto`
- **RPC**：
  - `NodeStream(stream NodeStreamRequest) returns (stream NodeStreamResponse)`
  - `GetUsers/GetConfig/GetInbounds/GetOutbounds/GetRouting`（当前为占位，返回空 payload）

**NodeStream 语义约定（当前实现）**：

- server -> node：
  - `UpdateEvent{kind, version, ts}`（用于通知节点“某类配置需要刷新”）
  - `Heartbeat{ts}`（服务端定时发心跳，占位）
- node -> server：
  - `PullAck{kind, version, ts}`（用于 sync 等待）
  - `Report{kind, payload, ts}`（占位日志）
  - `CommandResponse{...}`（占位日志）

### 10.4 鉴权 metadata 约定（服务端当前实现）

服务端会从 gRPC metadata 读取并校验：

- `x-node-id`：u64
- `x-node-token`：字符串（当 DB 中 `node_token` 为空时，当前实现会放行，但会 warn）
- `x-ts`：u64 秒级时间戳（允许 ±300s）
- `x-nonce`：随机串（服务端做重放保护）
- `x-sign`：HMAC-SHA256 签名（当 DB 中 `node_shared_secret` 为空时，当前实现会放行，但会 warn）

签名 base_string：

- NodePanelService（Ping）：`format!("{}:{}:{}", node_id, ts, nonce)`
- NodeControlService：
  - `format!("{}\n{}\n{}\n{}", node_id, token, ts, nonce)`

说明：这两处 base_string 当前实现不完全一致；后续在节点侧实现 gRPCPanel 前，建议统一协议（否则需要节点适配两套）。

### 10.5 运行/配置方法（环境变量）

#### 10.5.1 gRPC Server

- `GRPC_LISTEN_ADDR`：gRPC 内部监听地址（默认 `127.0.0.1:50051`）

#### 10.5.2 Admin HTTP

- `ADMIN_PORT`：admin 监听端口（默认 `667`）

#### 10.5.3 Client API HTTP

- `PORT`：client_api 监听端口（默认 `8080`）
- `CLIENT_API_PORT`：当需要给 TLS 入口推导后端端口时的替代变量（优先级可用于区分多进程）

#### 10.5.4 443 TLS 入口（SNI 转发）

- `ENABLE_TLS_ENTRY=1`：启用 TLS 入口（默认关闭）
- `TLS_ENTRY_ADDR`：TLS 入口监听地址（默认 `0.0.0.0:443`）
- `TLS_CERT_DIR`：证书目录（需要包含 `fullchain.pem` 与 `privkey.pem`）

后端转发目标（可选，通常不用配，已支持自动推导）：

- `TLS_GRPC_BACKEND_ADDR`
- `TLS_ADMIN_BACKEND_ADDR`
- `TLS_API_BACKEND_ADDR`

连接与超时控制：

- `TLS_MAX_CONNS`（默认 1024）
- `TLS_HANDSHAKE_TIMEOUT_MS`（默认 8000）
- `TLS_BACKEND_CONNECT_TIMEOUT_MS`（默认 3000）
- `TLS_SESSION_TIMEOUT_SECS`（默认 0=关闭）

#### 10.5.5 灰度/双栈开关

- DB：`admin_node_configs.node_comm_mode`：
  - `mqtt`：只走 MQTT
  - `grpc`：只走 gRPC（不 fallback）
  - `auto`：灰度策略（见下）
- `GRPC_GRAY_NODE_IDS`：逗号分隔的 node_id 列表，仅在 `node_comm_mode=auto` 时生效

### 10.6 测试与验证方法（推荐按顺序执行）

本节给出“最小可验证路径”，目标是验证：

- gRPC server 可用
- TLS 443 SNI 转发可用
- admin API 仍然可用
- 双栈选择逻辑生效

#### 10.6.1 编译检查

```bash
cargo check
```

#### 10.6.2 启动 admin 进程（包含 gRPC server + MQTT + 可选 TLS 入口）

示例（PowerShell）：

```powershell
$env:GRPC_LISTEN_ADDR = "127.0.0.1:50051"
$env:ADMIN_PORT = "667"

# 如需启用 443 入口：
# $env:ENABLE_TLS_ENTRY = "1"
# $env:TLS_CERT_DIR = "D:\path\to\certs"

cargo run
```

#### 10.6.3 启动 client_api（如果需要验证 api.<domain> 分流）

```powershell
$env:PORT = "8080"
cargo run --bin client_api
```

#### 10.6.4 验证 admin HTTP 仍可访问

（示例）

```bash
curl http://127.0.0.1:667/
```

#### 10.6.5 验证 gRPC（直连本机端口）

推荐使用 `grpcurl`（需要自行安装）：

```bash
grpcurl -plaintext 127.0.0.1:50051 list
```

鉴权 Ping（示例，签名仅演示，实际请按服务端实现生成 x-sign）：

```bash
grpcurl -plaintext \
  -H 'x-node-id: 1' \
  -H 'x-node-token: test' \
  -H 'x-ts: 1730000000' \
  -H 'x-nonce: n1' \
  -H 'x-sign: <hex-hmac>' \
  127.0.0.1:50051 nodepanel.NodePanel/Ping
```

#### 10.6.6 验证 TLS 443 SNI 转发（grpc/admin/api）

前置：

- `ENABLE_TLS_ENTRY=1`
- `TLS_CERT_DIR` 指向包含 `fullchain.pem` 和 `privkey.pem` 的目录

验证 SNI 到 admin（示例，使用 openssl）：

```bash
openssl s_client -connect 127.0.0.1:443 -servername admin.example.com -alpn http/1.1
```

验证 SNI 到 grpc（示例，使用 openssl）：

```bash
openssl s_client -connect 127.0.0.1:443 -servername grpc.example.com -alpn h2
```

说明：当前 TLS 入口是 TCP 转发层，具体 HTTP/gRPC 请求仍需要对应客户端（curl/grpcurl）。

#### 10.6.7 验证双栈选择逻辑（不需要节点升级）

由于当前节点侧尚未实现 gRPCPanel，因此仅能验证“选择逻辑 + fallback 行为”。

- 当 `node_comm_mode=mqtt`：所有 `publish_update_notification/wait_for_node_pull/query_*` 走 MQTT（现有行为）。
- 当 `node_comm_mode=grpc`：`publish_update_notification/wait_for_node_pull` 会尝试 gRPC；若节点未连接 gRPC stream，会返回错误（不会 fallback）。
- 当 `node_comm_mode=auto`：
  - 如果 node_id 在 `GRPC_GRAY_NODE_IDS` 中：优先 gRPC，失败 fallback MQTT
  - 如果不在：走 MQTT

#### 10.6.8 验证 MQTT（如需确认 MQTT TLS）

可复用仓库自带脚本：`README_MQTT_TEST.md` + `test_mqtt_tls.py`。

