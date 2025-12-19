// 管理接口 API 封装

import type {
  ApiResponse,
  NodeInfo,
  NodeConfig,
  User,
  OutboundConfig,
  RoutingConfig,
  RoutingRule,
  UserMapping,
  MaintenanceMode,
  UserLog,
  UdpLatencyResult,
} from './types';

const API_BASE = import.meta.env.VITE_API_BASE || '';

async function request<T>(
  url: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const response = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return response.json();
}

function buildQuery(params: Record<string, any>): string {
  const query = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null) {
      query.append(key, String(value));
    }
  });
  return query.toString();
}

// ========== 节点管理 ==========

export async function listNodes(): Promise<NodeInfo[]> {
  const response = await request<NodeInfo[]>('/api/admin/nodes');
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取节点列表失败');
}

export async function updateNodeMeta(
  nodeId: number,
  meta: { name?: string | null; region?: string | null; description?: string | null }
): Promise<void> {
  const response = await request(`/api/admin/nodes/${nodeId}/meta`, {
    method: 'PUT',
    body: JSON.stringify(meta),
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '更新节点信息失败');
  }
}

export async function getNodeConfig(nodeId: number): Promise<NodeConfig> {
  const query = buildQuery({ node_id: nodeId, act: 'config' });
  const response = await request<NodeConfig>(`/api/admin/query?${query}`);
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取节点配置失败');
}

// ========== 用户管理 ==========

export async function getUsers(nodeId: number): Promise<User[]> {
  const query = buildQuery({ node_id: nodeId, act: 'user' });
  const response = await request<User[]>(`/api/admin/query?${query}`);
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取用户列表失败');
}

export async function addUser(
  nodeId: number,
  user: { uuid: string; st?: number; dt?: number }
): Promise<User> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<User>(`/api/admin/user?${query}`, {
    method: 'POST',
    body: JSON.stringify(user),
  });
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '添加用户失败');
}

export async function updateUser(
  nodeId: number,
  userId: number,
  user: { uuid?: string; st?: number; dt?: number }
): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/user/${userId}?${query}`, {
    method: 'PUT',
    body: JSON.stringify(user),
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '更新用户失败');
  }
}

export async function deleteUser(nodeId: number, userId: number): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/user/${userId}?${query}`, {
    method: 'DELETE',
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '删除用户失败');
  }
}

// ========== 上游代理管理 ==========

export async function getOutbounds(
  nodeId: number
): Promise<{ outbounds: OutboundConfig[]; user_mapping: UserMapping }> {
  const query = buildQuery({ node_id: nodeId, act: 'outbound' });
  const response = await request<{
    outbounds: OutboundConfig[];
    user_mapping: UserMapping;
  }>(`/api/admin/query?${query}`);
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取上游代理列表失败');
}

export async function addOutbound(
  nodeId: number,
  outbound: { tag: string; protocol?: string; settings: any }
): Promise<OutboundConfig> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<OutboundConfig>(`/api/admin/outbound?${query}`, {
    method: 'POST',
    body: JSON.stringify(outbound),
  });
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '添加上游代理失败');
}

export async function updateOutbound(
  nodeId: number,
  tag: string,
  outbound: { protocol?: string; settings?: any }
): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/outbound/${tag}?${query}`, {
    method: 'PUT',
    body: JSON.stringify(outbound),
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '更新上游代理失败');
  }
}

export async function deleteOutbound(nodeId: number, tag: string): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/outbound/${tag}?${query}`, {
    method: 'DELETE',
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '删除上游代理失败');
  }
}

export async function queryUdpLatency(
  nodeId: number,
  outboundTag: string
): Promise<UdpLatencyResult> {
  const query = buildQuery({ node_id: nodeId, outbound_tag: outboundTag });
  const response = await request<UdpLatencyResult>(
    `/api/admin/outbound/udp_latency?${query}`
  );
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '查询 UDP 延迟失败');
}

// ========== 路由配置管理 ==========

export async function getRouting(nodeId: number): Promise<RoutingConfig> {
  const query = buildQuery({ node_id: nodeId, act: 'routing' });
  const response = await request<RoutingConfig>(`/api/admin/query?${query}`);
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取路由配置失败');
}

export async function updateRouting(
  nodeId: number,
  routing: {
    domain_strategy?: string;
    rules?: RoutingRule[];
    routing?: any;
  }
): Promise<RoutingConfig> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<RoutingConfig>(`/api/admin/routing?${query}`, {
    method: 'POST',
    body: JSON.stringify(routing),
  });
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '更新路由配置失败');
}

export async function addRoutingRule(
  nodeId: number,
  rule: RoutingRule
): Promise<RoutingConfig> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<RoutingConfig>(
    `/api/admin/routing/rule?${query}`,
    {
      method: 'POST',
      body: JSON.stringify(rule),
    }
  );
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '添加路由规则失败');
}

export async function updateRoutingRule(
  nodeId: number,
  index: number,
  rule: RoutingRule
): Promise<RoutingRule> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<RoutingRule>(
    `/api/admin/routing/rule/${index}?${query}`,
    {
      method: 'PUT',
      body: JSON.stringify(rule),
    }
  );
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '更新路由规则失败');
}

export async function deleteRoutingRule(
  nodeId: number,
  index: number
): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/routing/rule/${index}?${query}`, {
    method: 'DELETE',
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '删除路由规则失败');
  }
}

// ========== 用户映射管理 ==========

export async function addMapping(
  nodeId: number,
  mapping: { uuid: string; outbound_tag: string }
): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/mapping?${query}`, {
    method: 'POST',
    body: JSON.stringify(mapping),
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '添加用户映射失败');
  }
}

export async function updateMapping(
  nodeId: number,
  uuid: string,
  outboundTag: string
): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/mapping/${uuid}?${query}`, {
    method: 'PUT',
    body: JSON.stringify({ outbound_tag: outboundTag }),
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '更新用户映射失败');
  }
}

export async function deleteMapping(nodeId: number, uuid: string): Promise<void> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request(`/api/admin/mapping/${uuid}?${query}`, {
    method: 'DELETE',
  });
  if (response.msg !== 'ok') {
    throw new Error(response.error || '删除用户映射失败');
  }
}

// ========== 维护模式管理 ==========

export async function getMaintenanceMode(
  nodeId: number
): Promise<MaintenanceMode> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<MaintenanceMode>(
    `/api/admin/maintenance?${query}`
  );
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '获取维护模式失败');
}

export async function setMaintenanceMode(
  nodeId: number,
  enabled: boolean
): Promise<MaintenanceMode> {
  const query = buildQuery({ node_id: nodeId });
  const response = await request<MaintenanceMode>(
    `/api/admin/maintenance?${query}`,
    {
      method: 'POST',
      body: JSON.stringify({ enabled }),
    }
  );
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '设置维护模式失败');
}

// ========== 用户日志查询 ==========

export async function getUserLogs(
  nodeId: number,
  params: {
    uid: number;
    days?: number;
    limit?: number;
    event?: string;
  }
): Promise<UserLog[]> {
  const query = buildQuery({ node_id: nodeId, ...params, act: 'user_logs' });
  const response = await request<UserLog[]>(`/api/admin/query?${query}`);
  if (response.msg === 'ok' && response.data) {
    return response.data;
  }
  throw new Error(response.error || '查询用户日志失败');
}

