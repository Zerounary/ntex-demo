# 链路管理重构总结

_日期：2026-01-11_

## 后端改动

1. **`admin_chains` 表结构与模型**
   - `admin_chains.id` 改为自增 `BIGINT`，同时完全删除 `uuid` 列。
   - 更新 Rust 端的 `ChainDefinition` 以及所有相关 DTO，仅暴露数值型 `id`，不再序列化 `uuid`。

2. **链路配置存储（`admin_config.rs`）**
   - 新增单链路的 `upsert_chain` / `delete_chain` 方法，返回保存后的记录及序列化路由。
   - `get_chains` 仍负责反序列化 JSON routes，调用方统一使用数字 ID。

3. **管理端 HTTP Handler / 路由**
   - 新增接口：
     - `POST /api/admin/chains` → 按 `id` 新增或更新单条链路。
     - `DELETE /api/admin/chains/{id}` → 删除单条链路并触发清理。
     - `POST /api/admin/chains/apply` → 每次只应用一条链路（参数 `{ chain_id, base_port? }`）。
   - 移除旧的批量 `update_chains` 接口。

4. **链路应用语义（`chain_ops.rs`）**
   - 链路检索统一使用数值 ID。
   - 透明代理入站 **仅为倒数第二及之前的节点** 创建；最后一个节点的认证交由节点自身处理。
   - 清理逻辑针对 `chain_{id}_{hop}` 形式的标签。

5. **Session 启动流程（`session_start_v2`）**
   - 请求 DTO 改为 `chain_id: Option<i64>`，不再接受字符串 UUID。
   - 最后一跳节点不再注入用户/映射配置，保留节点自身的认证逻辑。

## 前端改动（管理后台）

1. **API 客户端（`admin.ts` / `types.ts`）**
   - 新增强类型的 `getChains`、`upsertChain`、`deleteChain`、`applyChain`，全部使用数值型链路 ID。
   - 移除批量 `updateChains` 以及 `ChainDefinition` 中的全部 `uuid` 字段。

2. **`ChainManagement.vue`**
   - 组件内部改为管理 `activeChainId: number | null`，临时未保存的链路使用负数 ID。
   - 保存 / 应用操作仅针对当前选中链路，调用新的单链 API。
   - 删除 UUID 展示、复制/重生按钮，以及批量保存/应用逻辑。
   - “Apply” 在链路保存前（ID ≤ 0）保持禁用。

3. **相关页面**
   - `ChainManagement` 被 `pages/chains/index.vue` 与 `NodeChainsEditor.vue` 共用，不再传递额外的 `node_id` 或 UUID。

## 其他说明

- Flow 画布工具（`pages/flow`、`stores/flow.ts`）仍保留自身的本地 `uuid` 概念，用于可视化编辑，与 admin 链路 CRUD 解耦，因此暂未调整。
- 应用或清理链路后仍会发送 MQTT 通知，以保持各节点配置状态同步。
