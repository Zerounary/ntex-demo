// API 类型定义

export interface ApiResponse<T = any> {
  msg: 'ok' | 'error';
  data?: T;
  error?: string;
  message?: string;
}

export interface NodeInfo {
  node_id: number;
  node_type: string;
  node_speed_limit: number;
  traffic_rate: number;
  sort: number;
  maintenance_mode: boolean;
  cpu_usage?: number;
  mem_usage?: number;
  disk_usage?: number;
  uptime?: number;
  online_user_count?: number;
  cpu_threads?: number;
  mem_total?: number;
  disk_total?: number;
  network_interfaces?: any;
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

