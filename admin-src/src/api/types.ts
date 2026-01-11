// API 类型定义

export interface ApiResponse<T = any> {
  msg: 'ok' | 'error';
  data?: T;
  error?: string;
  message?: string;
}

export interface NodeNetworkInterface {
  name: string;
  bytes_recv: number;
  bytes_sent: number;
  packets_recv: number;
  packets_sent: number;
  speed: number;
}

export interface NodeInfo {
  node_id: number;
  name?: string | null;
  region?: string | null;
  description?: string | null;
  node_type: string;
  node_speed_limit: number;
  traffic_rate: number;
  sort: number;
  maintenance_mode: boolean;
  is_online?: boolean;
  last_seen_at?: string | null;
  cpu_usage?: number;
  mem_usage?: number;
  disk_usage?: number;
  uptime?: number;
  online_user_count?: number;
  cpu_threads?: number;
  mem_total?: number;
  disk_total?: number;
  public_ip?: string | null;
  network_interfaces?: NodeNetworkInterface[];
  created_at: string;
  updated_at: string;
}

export interface NodeConfig {
  node_id: number;
  node_type: string;
  node_speed_limit: number;
  traffic_rate: number;
  sort: number;
  inbounds: any[];
  cpu_threads?: number;
  mem_total?: number;
  disk_total?: number;
}

export interface User {
  id: number;
  uuid: string;
  st: number; // 限速值（Mbps）
  dt: number; // 设备限制
}

export interface OutboundConfig {
  tag: string;
  protocol: string;
  settings: any;
  stream_settings?: any;
}

export interface InboundConfig {
  tag: string;
  protocol: string;
  port: number;
  listen?: string | null;
  settings: any;
  stream_settings?: any;
  sniffing?: any;
}

export interface RoutingRule {
  type: string;
  outbound_tag?: string;
  domain?: string[];
  ip?: string[];
  port?: string;
  network?: string;
  source?: string[];
  protocol?: string[];
}

export interface RoutingConfig {
  domainStrategy: string;
  rules: RoutingRule[];
}

export interface ChainRouteEntry {
  id: string;
  order: number;
  fromNodeId: number;
  toNodeId: number;
  mode: string;
  remark?: string;
}

export interface ChainDefinition {
  id: number;
  name: string;
  protocol: string;
  routes: ChainRouteEntry[];
  createdAt: string;
  updatedAt: string;
  description?: string;
}

export interface ApplyChainRequest {
  chain_id: number;
  base_port?: number;
}

export interface ApplyChainNodeResult {
  node_id: number;
  tag: string;
  status: 'ok' | 'error';
  error?: string;
}

export interface ApplyChainResult {
  chain_id: number;
  applied_nodes: ApplyChainNodeResult[];
}

export interface UserMapping {
  [uuid: string]: string; // uuid -> outbound_tag
}

export interface MaintenanceMode {
  maintenance_mode: boolean;
  description?: string;
}

export interface UserLog {
  uid: number;
  event: string;
  timestamp: string;
  [key: string]: any;
}

export interface UdpLatencyResult {
  latency?: number;
  error?: string;
}

