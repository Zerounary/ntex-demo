#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
简单的本地 API 服务器，为 XrayR 提供用户信息和上游代理配置
支持热重载：修改配置后无需重启 XrayR
支持 HTTP 和 MQTT 两种协议
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import threading
import time

# MQTT 支持（可选，需要安装 paho-mqtt: pip install paho-mqtt）
try:
    import paho.mqtt.client as mqtt
    MQTT_AVAILABLE = True
except ImportError:
    MQTT_AVAILABLE = False
    print("⚠️  MQTT 支持未启用，需要安装 paho-mqtt: pip install paho-mqtt")

# 用户配置 - VMess 用户（基于 UUID 区分）
# V2RaySocks API 格式：uuid=用户UUID, st=限速(Mbps), dt=设备限制
USERS = [
    {
        "id": 1,
        "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",  # UUID
        "st": 5,  # 限速 5 Mbps
        "dt": 0  # 设备限制
    },
    {
        "id": 2,
        "uuid": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
        "st": 1,  # 限速 1 Mbps
        "dt": 0
    },
    {
        "id": 3,
        "uuid": "c3d4e5f6-a7b8-9012-cdef-123456789012",
        "st": 1,  # 限速 1 Mbps
        "dt": 0
    },
    {
        "id": 4,
        "uuid": "d4e5f6a7-b8c9-0123-def0-234567890123",
        "st": 1,  # 限速 1 Mbps
        "dt": 0
    },
    {
        "id": 5,
        "uuid": "e5f6a7b8-c9d0-1234-ef01-345678901234",
        "st": 1,  # 限速 1 Mbps
        "dt": 0
    },
    {
        "id": 6,
        "uuid": "f6a7b8c9-d0e1-2345-f012-456789012345",
        "st": 1,  # 限速 1 Mbps
        "dt": 0
    }
]

# 用户到上游代理的映射（基于 UUID）
USER_OUTBOUND_MAPPING = {
    "a1b2c3d4-e5f6-7890-abcd-ef1234567890": "ss_1",
    "b2c3d4e5-f6a7-8901-bcde-f12345678901": "ss_2",
    "c3d4e5f6-a7b8-9012-cdef-123456789012": "ss_3",
    "d4e5f6a7-b8c9-0123-def0-234567890123": "ss_1",
    "e5f6a7b8-c9d0-1234-ef01-345678901234": "ss_2"
}

# 上游代理配置 - Shadowsocks 上游服务器
# 修改这里的配置后，XrayR 会在下次更新周期（默认 60 秒）自动重载
OUTBOUND_CONFIGS = [
    {
        "tag": "block",
        "protocol": "blackhole",
        "settings": {
            "response": {
                "type": "http"
            }
        }
    },
    {
        "tag": "direct",
        "protocol": "freedom",
        "settings": {}
    },
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
  },
  {
    "tag": "ss_2",
    "protocol": "shadowsocks",
    "settings": {
      "servers": [
        {
          "address": "65.49.212.165",
          "port": 19166,
          "method": "aes-256-gcm",
          "password": "bxaeWJ4Kf9ZL59R3"
        }
      ]
    }
  },
  {
    "tag": "ss_3",
    "protocol": "shadowsocks",
    "settings": {
      "servers": [
        {
          "address": "65.49.212.165",
          "port": 19166,
          "method": "aes-256-gcm",
          "password": "bxaeWJ4Kf9ZL59R3"
        }
      ]
    }
  },
  # 本地回环自测：vmess outbound 指向本机 vmess inbound（UDP 探测自测）
  {
    "tag": "vmess_loopback",
    "protocol": "vmess",
    "settings": {
      "vnext": [
        {
          "address": "127.0.0.1",
          "port": 10086,
          "users": [
            {
              "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
              "alterId": 0,
              "email": "t@t.tt",
              "security": "auto"
            }
          ]
        }
      ]
    },
    "streamSettings": {
      "network": "tcp"
    }
  }
]

# 节点配置 - VMess 协议监听 10086 端口
NODE_INFO = {
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

# Routing 配置 - 默认内置一组“类似 GFW 拦截”的规则示例
# 说明：
# 1) customRouter 不解析 geosite/geoip，这里全部用显式域名/关键字/IP/CIDR
# 2) domain:xxx 会匹配 xxx 及其子域名；keyword: 会兜底更多包含该关键字的域名
# 3) 如需追加/删除，请通过管理接口更新（支持 MQTT 实时下发）
ROUTING_CONFIG = {
    "domainStrategy": "AsIs",
    "rules": [
    ]
}

# 全局 MQTT 处理器引用（用于推送更新通知）
mqtt_handler_global = None

# 配置锁（确保多线程安全）
config_lock = threading.Lock()

# 日志查询响应等待字典（用于异步 MQTT 响应）
log_query_responses = {}
log_query_lock = threading.Lock()

# UDP 探测配置与缓存
UDP_PROBE_ENABLED = True
UDP_PROBE_INTERVAL = 60  # 秒
UDP_PROBE_TIMEOUT = 8    # 秒
udp_probe_results = {}   # {tag: {"latency_ms": float, "timestamp": int}}

# 维护模式配置
MAINTENANCE_MODE = False  # 维护模式标志：开启时跳过新用户添加，仅允许已存在用户


class APIHandler(BaseHTTPRequestHandler):
    def _send_json_response(self, data, status_code=200):
        """发送 JSON 响应"""
        self.send_response(status_code)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False).encode())
    
    def _read_json_body(self):
        """读取 JSON 请求体"""
        content_length = int(self.headers.get('Content-Length', 0))
        if content_length == 0:
            return None
        body = self.rfile.read(content_length)
        try:
            return json.loads(body.decode())
        except:
            return None
    
    def do_GET(self):
        global mqtt_handler_global
        from urllib.parse import urlparse, parse_qs
        parsed = urlparse(self.path)
        params = parse_qs(parsed.query)

        # V2RaySocks API: ?act=user&node_type=vless
        if 'act' in params:
            act = params['act'][0]

            if act == 'user':
                # 返回用户列表
                # 如果维护模式开启，记录日志但不影响返回
                if MAINTENANCE_MODE:
                    print(f"⚠️  [维护模式] 维护模式已启用，新用户添加将被跳过")
                
                self._send_json_response({
                    "msg": "ok",
                    "data": USERS
                })
                return

            elif act == 'config':
                # 返回节点配置
                self._send_json_response({
                    "msg": "ok",
                    "data": NODE_INFO
                })
                return

            elif act == 'outbound':
                # 返回上游代理配置（新增接口）
                self._send_json_response({
                    "msg": "ok",
                    "data": {
                        "outbounds": OUTBOUND_CONFIGS,
                        "user_mapping": USER_OUTBOUND_MAPPING
                    }
                })
                return

            elif act == 'routing':
                # 返回路由配置（新增接口）
                self._send_json_response({
                    "msg": "ok",
                    "data": ROUTING_CONFIG
                })
                return
            
            elif act == 'maintenance':
                # 返回维护模式状态
                self._send_json_response({
                    "msg": "ok",
                    "data": {
                        "maintenance_mode": MAINTENANCE_MODE,
                        "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
                    }
                })
                return
            
            elif act == 'user_logs':
                # 查询用户日志（通过 MQTT 从节点获取）
                uid = params.get('uid', [None])[0]
                days = params.get('days', ['7'])[0]
                event = params.get('event', [None])[0]
                limit = params.get('limit', ['100'])[0]
                
                if not uid:
                    self._send_json_response({
                        "msg": "error",
                        "error": "uid 参数是必需的"
                    }, 400)
                    return
                
                try:
                    uid_int = int(uid)
                    days_int = int(days) if days else 7
                    limit_int = int(limit) if limit else 100
                except ValueError:
                    self._send_json_response({
                        "msg": "error",
                        "error": "无效的参数格式"
                    }, 400)
                    return
                
                # 构建查询参数
                query_params = {
                    "uid": uid_int,
                    "days": days_int,
                    "limit": limit_int
                }
                if event:
                    query_params["event"] = event
                
                # 通过 MQTT 查询节点日志
                if mqtt_handler_global is None:
                    self._send_json_response({
                        "msg": "error",
                        "error": "MQTT 未连接，无法查询日志"
                    }, 503)
                    return
                
                node_id = NODE_INFO.get("node_id", 41)
                result = mqtt_handler_global.query_node_logs(node_id, query_params, timeout=15)
                
                if result is None:
                    self._send_json_response({
                        "msg": "error",
                        "error": "查询日志超时或失败"
                    }, 504)
                    return
                
                # 返回查询结果
                self._send_json_response(result)
                return

        # 兼容旧的 API 路径
        if self.path.startswith('/api/v1/server/UniProxy/user'):
            self._send_json_response({
                "msg": "ok",
                "data": USERS
            })

        elif self.path.startswith('/api/v1/server/UniProxy/config'):
            self._send_json_response({
                "msg": "ok",
                "data": NODE_INFO
            })

        elif self.path.startswith('/api/v1/server/UniProxy/outbound'):
            # 新增：上游代理配置接口
            self._send_json_response({
                "msg": "ok",
                "data": {
                    "outbounds": OUTBOUND_CONFIGS,
                    "user_mapping": USER_OUTBOUND_MAPPING
                }
            })

        elif self.path.startswith('/api/v1/server/UniProxy/routing'):
            # 新增：路由配置接口
            self._send_json_response({
                "msg": "ok",
                "data": ROUTING_CONFIG
            })
        
        elif self.path.startswith('/api/v1/server/UniProxy/user_logs'):
            # 兼容路径：查询用户日志
            from urllib.parse import urlparse, parse_qs
            parsed = urlparse(self.path)
            params = parse_qs(parsed.query)
            
            uid = params.get('uid', [None])[0]
            days = params.get('days', ['7'])[0]
            event = params.get('event', [None])[0]
            limit = params.get('limit', ['100'])[0]
            
            if not uid:
                self._send_json_response({
                    "msg": "error",
                    "error": "uid 参数是必需的"
                }, 400)
                return
            
            try:
                uid_int = int(uid)
                days_int = int(days) if days else 7
                limit_int = int(limit) if limit else 100
            except ValueError:
                self._send_json_response({
                    "msg": "error",
                    "error": "无效的参数格式"
                }, 400)
                return
            
            # 构建查询参数
            query_params = {
                "uid": uid_int,
                "days": days_int,
                "limit": limit_int
            }
            if event:
                query_params["event"] = event
            
            # 通过 MQTT 查询节点日志
            if mqtt_handler_global is None:
                self._send_json_response({
                    "msg": "error",
                    "error": "MQTT 未连接，无法查询日志"
                }, 503)
                return
            
            node_id = NODE_INFO.get("node_id", 41)
            result = mqtt_handler_global.query_node_logs(node_id, query_params, timeout=15)
            
            if result is None:
                self._send_json_response({
                    "msg": "error",
                    "error": "查询日志超时或失败"
                }, 504)
                return
            
            # 返回查询结果
            self._send_json_response(result)

        elif self.path.startswith('/api/admin/outbound/udp_latency'):
            # 主动查询某个 outbound 的 UDP 延迟
            print(f"🌐 [HTTP API] 收到 UDP 延迟查询请求: {self.path}")
            
            if mqtt_handler_global is None:
                print(f"❌ [HTTP API] MQTT 未连接，无法查询 UDP 延迟")
                self._send_json_response({
                    "msg": "error",
                    "error": "MQTT 未连接，无法查询 UDP 延迟"
                }, 503)
                return

            from urllib.parse import urlparse, parse_qs
            parsed = urlparse(self.path)
            params = parse_qs(parsed.query)
            outbound_tag = params.get('outbound_tag', [None])[0]
            node_id = params.get('node_id', [None])[0]

            print(f"📋 [HTTP API] 解析参数: outbound_tag={outbound_tag}, node_id={node_id}")

            if not outbound_tag:
                print(f"❌ [HTTP API] outbound_tag 参数缺失")
                self._send_json_response({
                    "msg": "error",
                    "error": "outbound_tag 参数是必需的"
                }, 400)
                return

            try:
                node_id_int = int(node_id) if node_id else NODE_INFO.get("node_id", 41)
            except ValueError:
                print(f"❌ [HTTP API] 无效的 node_id: {node_id}")
                self._send_json_response({
                    "msg": "error",
                    "error": "无效的 node_id"
                }, 400)
                return

            print(f"🚀 [HTTP API] 调用 query_udp_latency: node_id={node_id_int}, outbound_tag={outbound_tag}")
            result = mqtt_handler_global.query_udp_latency(node_id_int, outbound_tag, timeout=UDP_PROBE_TIMEOUT)
            
            if result is None:
                print(f"❌ [HTTP API] UDP 延迟查询失败或超时")
                self._send_json_response({
                    "msg": "error",
                    "error": "查询 UDP 延迟超时或失败"
                }, 504)
                return

            print(f"✅ [HTTP API] UDP 延迟查询成功: {json.dumps(result)}")

            # 缓存结果，供定时查询使用
            udp_probe_results[outbound_tag] = {
                "latency_ms": result.get("latency_ms", None),
                "timestamp": int(time.time())
            }

            self._send_json_response({
                "msg": "ok",
                "data": result
            })

        elif self.path == '/api/admin/maintenance':
            # GET: 获取维护模式状态
            self._send_json_response({
                "msg": "ok",
                "data": {
                    "maintenance_mode": MAINTENANCE_MODE,
                    "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
                }
            })

        else:
            self.send_response(404)
            self.end_headers()
    
    def do_POST(self):
        global MAINTENANCE_MODE
        
        # 处理流量上报（忽略）
        if self.path.startswith('/api/v1/server/UniProxy/push'):
            self._send_json_response({"msg": "ok"})
        
        # ========== 管理接口：用户管理 ==========
        elif self.path == '/api/admin/user':
            # 添加用户（增量添加：如果用户已存在则跳过）
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            uuid = data.get("uuid")
            if not uuid:
                self._send_json_response({"msg": "error", "error": "uuid 是必需的"}, 400)
                return
            
            with config_lock:
                # 检查用户是否已存在（基于 UUID）
                existing_user = None
                for user in USERS:
                    if user.get("uuid") == uuid:
                        existing_user = user
                        break
                
                if existing_user:
                    # 用户已存在，返回现有用户信息
                    self._send_json_response({
                        "msg": "ok",
                        "data": existing_user,
                        "message": "用户已存在，未添加"
                    })
                    return
                
                # 检查维护模式
                if MAINTENANCE_MODE:
                    self._send_json_response({
                        "msg": "error",
                        "error": "维护模式已启用，无法添加新用户。仅允许已存在用户连接。",
                        "maintenance_mode": True
                    }, 503)
                    print(f"⚠️  [维护模式] 拒绝添加新用户: uuid={uuid} (维护模式已启用)")
                    return
                
                # 用户不存在，添加新用户
                new_id = max([u["id"] for u in USERS], default=0) + 1
                user = {
                    "id": new_id,
                    "uuid": uuid,
                    "st": data.get("st", 1),
                    "dt": data.get("dt", 0)
                }
                
                USERS.append(user)
            
            # 推送更新通知
            _notify_update('user')
            
            self._send_json_response({
                "msg": "ok",
                "data": user,
                "message": "用户添加成功"
            })
        
        elif self.path.startswith('/api/admin/user/'):
            # 更新用户
            try:
                user_id = int(self.path.split('/')[-1])
            except:
                self._send_json_response({"msg": "error", "error": "无效的用户 ID"}, 400)
                return
            
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            with config_lock:
                found = False
                for i, user in enumerate(USERS):
                    if user["id"] == user_id:
                        # 更新用户
                        if "uuid" in data:
                            user["uuid"] = data["uuid"]
                        if "st" in data:
                            user["st"] = data["st"]
                        if "dt" in data:
                            user["dt"] = data["dt"]
                        found = True
                        break
                
                if not found:
                    self._send_json_response({"msg": "error", "error": "用户不存在"}, 404)
                    return
            
            # 推送更新通知
            _notify_update('user')
            
            self._send_json_response({"msg": "ok"})
        
        # ========== 管理接口：上游代理管理 ==========
        elif self.path == '/api/admin/outbound':
            # 添加上游代理
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            tag = data.get("tag")
            if not tag:
                self._send_json_response({"msg": "error", "error": "tag 是必需的"}, 400)
                return
            
            with config_lock:
                # 检查 tag 是否已存在
                for outbound in OUTBOUND_CONFIGS:
                    if outbound.get("tag") == tag:
                        error_msg = "tag " + tag + " 已存在"
                        self._send_json_response({"msg": "error", "error": error_msg}, 400)
                        return
                
                outbound_config = {
                    "tag": tag,
                    "protocol": data.get("protocol", "shadowsocks"),
                    "settings": data.get("settings", {})
                }
                OUTBOUND_CONFIGS.append(outbound_config)
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({
                "msg": "ok",
                "data": outbound_config
            })
        
        elif self.path.startswith('/api/admin/outbound/'):
            # 更新上游代理
            try:
                tag = self.path.split('/')[-1]
            except:
                self._send_json_response({"msg": "error", "error": "无效的 tag"}, 400)
                return
            
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            with config_lock:
                found = False
                for outbound in OUTBOUND_CONFIGS:
                    if outbound.get("tag") == tag:
                        # 更新配置
                        if "protocol" in data:
                            outbound["protocol"] = data["protocol"]
                        if "settings" in data:
                            outbound["settings"] = data["settings"]
                        found = True
                        break
                
                if not found:
                    self._send_json_response({"msg": "error", "error": "上游代理不存在"}, 404)
                    return
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({"msg": "ok"})
        
        # ========== 管理接口：路由配置管理 ==========
        elif self.path == '/api/admin/routing':
            # 更新整个路由配置
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            with config_lock:
                # 更新路由配置
                if "domainStrategy" in data:
                    ROUTING_CONFIG["domainStrategy"] = data["domainStrategy"]
                if "rules" in data:
                    ROUTING_CONFIG["rules"] = data["rules"]
                # 如果提供了完整的 routing 配置，直接替换
                if "routing" in data:
                    ROUTING_CONFIG.update(data["routing"])
            
            # 推送更新通知
            _notify_update('routing')
            
            self._send_json_response({
                "msg": "ok",
                "data": ROUTING_CONFIG
            })
        
        elif self.path == '/api/admin/routing/rule':
            # 添加路由规则
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            with config_lock:
                if "rules" not in ROUTING_CONFIG:
                    ROUTING_CONFIG["rules"] = []
                ROUTING_CONFIG["rules"].append(data)
            
            # 推送更新通知
            _notify_update('routing')
            
            self._send_json_response({
                "msg": "ok",
                "data": ROUTING_CONFIG
            })
        
        elif self.path.startswith('/api/admin/routing/rule/'):
            # 更新或删除路由规则（通过索引）
            try:
                rule_index = int(self.path.split('/')[-1])
            except:
                self._send_json_response({"msg": "error", "error": "无效的规则索引"}, 400)
                return
            
            if self.command == 'DELETE':
                # 删除规则
                with config_lock:
                    if "rules" in ROUTING_CONFIG and 0 <= rule_index < len(ROUTING_CONFIG["rules"]):
                        deleted_rule = ROUTING_CONFIG["rules"].pop(rule_index)
                        _notify_update('routing')
                        self._send_json_response({
                            "msg": "ok",
                            "data": {"deleted_rule": deleted_rule}
                        })
                    else:
                        self._send_json_response({"msg": "error", "error": "规则索引超出范围"}, 404)
                return
            
            # 更新规则
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            with config_lock:
                if "rules" in ROUTING_CONFIG and 0 <= rule_index < len(ROUTING_CONFIG["rules"]):
                    ROUTING_CONFIG["rules"][rule_index].update(data)
                    _notify_update('routing')
                    self._send_json_response({
                        "msg": "ok",
                        "data": ROUTING_CONFIG["rules"][rule_index]
                    })
                else:
                    self._send_json_response({"msg": "error", "error": "规则索引超出范围"}, 404)
        
        # ========== 管理接口：维护模式管理 ==========
        elif self.path == '/api/admin/maintenance':
            # 设置维护模式
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            enabled = data.get("enabled", False)
            
            with config_lock:
                old_mode = MAINTENANCE_MODE
                MAINTENANCE_MODE = enabled
            
            mode_str = "启用" if enabled else "禁用"
            print(f"🔧 [维护模式] {mode_str}维护模式 (之前: {'启用' if old_mode else '禁用'})")
            
            if enabled:
                print(f"⚠️  [维护模式] 维护模式已启用：新用户添加将被跳过，仅允许已存在用户连接")
            else:
                print(f"✅ [维护模式] 维护模式已禁用：允许新用户添加和连接")
            
            self._send_json_response({
                "msg": "ok",
                "data": {
                    "maintenance_mode": MAINTENANCE_MODE,
                    "previous_mode": old_mode,
                    "message": f"维护模式已{mode_str}"
                }
            })
        
        # ========== 管理接口：用户映射管理 ==========
        elif self.path == '/api/admin/mapping':
            # 添加用户映射
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            uuid = data.get("uuid")
            outbound_tag = data.get("outbound_tag")
            
            if not uuid or not outbound_tag:
                self._send_json_response({"msg": "error", "error": "uuid 和 outbound_tag 都是必需的"}, 400)
                return
            
            with config_lock:
                USER_OUTBOUND_MAPPING[uuid] = outbound_tag
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({
                "msg": "ok",
                "data": {uuid: outbound_tag}
            })
        
        else:
            self.send_response(404)
            self.end_headers()
    
    def do_PUT(self):
        # PUT 方法处理更新操作
        
        # ========== 管理接口：更新用户映射 ==========
        if self.path.startswith('/api/admin/mapping/'):
            # 更新用户映射
            try:
                uuid = self.path.split('/')[-1]
            except:
                self._send_json_response({"msg": "error", "error": "无效的 uuid"}, 400)
                return
            
            data = self._read_json_body()
            if not data:
                self._send_json_response({"msg": "error", "error": "无效的 JSON 数据"}, 400)
                return
            
            outbound_tag = data.get("outbound_tag")
            if not outbound_tag:
                self._send_json_response({"msg": "error", "error": "outbound_tag 是必需的"}, 400)
                return
            
            with config_lock:
                # 检查映射是否存在
                if uuid not in USER_OUTBOUND_MAPPING:
                    self._send_json_response({"msg": "error", "error": "映射不存在"}, 404)
                    return
                
                # 更新映射
                old_tag = USER_OUTBOUND_MAPPING[uuid]
                USER_OUTBOUND_MAPPING[uuid] = outbound_tag
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({
                "msg": "ok",
                "data": {
                    "uuid": uuid,
                    "old_outbound_tag": old_tag,
                    "new_outbound_tag": outbound_tag
                }
            })
        else:
            # 其他 PUT 操作通过 POST 处理
            self.do_POST()
    
    def do_DELETE(self):
        # ========== 管理接口：删除用户 ==========
        if self.path.startswith('/api/admin/user/'):
            try:
                user_id = int(self.path.split('/')[-1])
            except:
                self._send_json_response({"msg": "error", "error": "无效的用户 ID"}, 400)
                return
            
            with config_lock:
                found = False
                for i, user in enumerate(USERS):
                    if user["id"] == user_id:
                        deleted_user = USERS.pop(i)
                        found = True
                        # 同时删除映射
                        uuid = deleted_user.get("uuid")
                        if uuid and uuid in USER_OUTBOUND_MAPPING:
                            del USER_OUTBOUND_MAPPING[uuid]
                        break
                
                if not found:
                    self._send_json_response({"msg": "error", "error": "用户不存在"}, 404)
                    return
            
            # 推送更新通知
            _notify_update('user')
            _notify_update('outbound')  # 因为映射也变了
            
            self._send_json_response({"msg": "ok"})
        
        # ========== 管理接口：删除上游代理 ==========
        elif self.path.startswith('/api/admin/outbound/'):
            try:
                tag = self.path.split('/')[-1]
            except:
                self._send_json_response({"msg": "error", "error": "无效的 tag"}, 400)
                return
            
            with config_lock:
                found = False
                for i, outbound in enumerate(OUTBOUND_CONFIGS):
                    if outbound.get("tag") == tag:
                        OUTBOUND_CONFIGS.pop(i)
                        found = True
                        # 删除所有使用该 outbound 的映射
                        uuids_to_remove = [uuid for uuid, ot in USER_OUTBOUND_MAPPING.items() if ot == tag]
                        for uuid in uuids_to_remove:
                            del USER_OUTBOUND_MAPPING[uuid]
                        break
                
                if not found:
                    self._send_json_response({"msg": "error", "error": "上游代理不存在"}, 404)
                    return
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({"msg": "ok"})
        
        # ========== 管理接口：删除路由规则 ==========
        elif self.path.startswith('/api/admin/routing/rule/'):
            # 按索引删除 routing 规则
            try:
                rule_index = int(self.path.split('/')[-1])
            except:
                self._send_json_response({"msg": "error", "error": "无效的规则索引"}, 400)
                return
            
            with config_lock:
                if "rules" in ROUTING_CONFIG and 0 <= rule_index < len(ROUTING_CONFIG["rules"]):
                    deleted_rule = ROUTING_CONFIG["rules"].pop(rule_index)
                    _notify_update('routing')
                    self._send_json_response({
                        "msg": "ok",
                        "data": {"deleted_rule": deleted_rule}
                    })
                else:
                    self._send_json_response({"msg": "error", "error": "规则索引超出范围"}, 404)
            return
        
        # ========== 管理接口：删除用户映射 ==========
        elif self.path.startswith('/api/admin/mapping/'):
            try:
                uuid = self.path.split('/')[-1]
            except:
                self._send_json_response({"msg": "error", "error": "无效的 uuid"}, 400)
                return
            
            with config_lock:
                if uuid in USER_OUTBOUND_MAPPING:
                    del USER_OUTBOUND_MAPPING[uuid]
                else:
                    self._send_json_response({"msg": "error", "error": "映射不存在"}, 404)
                    return
            
            # 推送更新通知
            _notify_update('outbound')
            
            self._send_json_response({"msg": "ok"})
        
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        # 简化日志输出
        print("[HTTP] " + self.command + " " + self.path)


# MQTT 处理器
class MQTTHandler:
    def __init__(self, broker_host='127.0.0.1', broker_port=1883, username=None, password=None):
        if not MQTT_AVAILABLE:
            raise ImportError("paho-mqtt 未安装，无法使用 MQTT 功能")
        
        self.broker_host = broker_host
        self.broker_port = broker_port
        self.client = mqtt.Client()
        
        if username and password:
            self.client.username_pw_set(username, password)
        
        self.client.on_connect = self.on_connect
        self.client.on_message = self.on_message
        
    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print(f"✅ MQTT 已连接到 {self.broker_host}:{self.broker_port}")
            # 订阅所有节点请求主题
            client.subscribe("xrayr/node/+/request/+")
            print("📡 已订阅 MQTT 主题: xrayr/node/+/request/+")
            # 订阅所有节点响应主题（用于接收日志查询结果）
            client.subscribe("xrayr/node/+/response/+")
            print("📡 已订阅 MQTT 响应主题: xrayr/node/+/response/+")
        else:
            print(f"❌ MQTT 连接失败，错误代码: {rc}")
    
    def on_message(self, client, userdata, msg):
        try:
            # 解析主题: xrayr/node/{node_id}/request/{request_id} 或 xrayr/node/{node_id}/response/{request_id}
            topic_parts = msg.topic.split('/')
            if len(topic_parts) != 5:
                print(f"⚠️  无效的 MQTT 主题格式: {msg.topic}")
                return
            
            node_id = topic_parts[2]
            topic_type = topic_parts[3]  # 'request' 或 'response'
            request_id = topic_parts[4]
            
            # 如果是响应消息（来自节点的日志查询结果）
            if topic_type == 'response':
                self.handle_response(node_id, request_id, msg.payload)
                return
            
            # 处理请求消息
            # 解析请求
            request_data = json.loads(msg.payload.decode())
            action = request_data.get('action')
            token = request_data.get('token')
            node_type = request_data.get('node_type', '')
            
            print(f"[MQTT] 收到请求: node_id={node_id}, action={action}, request_id={request_id}")
            
            # 如果是上报操作，打印完整消息内容
            if action in ['submit', 'nodestatus', 'onlineusers', 'illegal', 'outbound_failure', 'outbound_recovery', 'outbound_latency', 'outbound_latencies']:
                print(f"📊 [上报消息] action={action}, node_id={node_id}")
                print(f"   完整消息内容: {json.dumps(request_data, indent=2, ensure_ascii=False)}")
            
            # 验证 token（这里简单验证，实际应该更严格）
            # if token != "123":  # 可以根据需要验证
            #     print(f"⚠️  无效的 token: {token}")
            #     return
            
            # 处理请求
            response = self.handle_request(action, request_data)
            
            # 如果返回 None，表示应该由节点端处理，管理端不发送响应
            if response is None:
                print(f"[MQTT] 请求 {action} 应该由节点端处理，管理端忽略")
                return
            
            # 发送响应
            response_topic = f"xrayr/node/{node_id}/response/{request_id}"
            response_json = json.dumps(response)
            client.publish(response_topic, response_json, qos=1)
            print(f"[MQTT] 已发送响应: {response_topic}")
            
        except Exception as e:
            print(f"❌ 处理 MQTT 消息时出错: {e}")
    
    def handle_response(self, node_id, request_id, payload):
        """处理来自节点的响应消息（用于日志查询）"""
        try:
            print(f"📥 [MQTT Response Handler] 收到响应: node_id={node_id}, request_id={request_id}")
            response_data = json.loads(payload.decode())
            print(f"📥 [MQTT Response Handler] 解析响应数据: {json.dumps(response_data, indent=2, ensure_ascii=False)}")
            
            log_query_lock.acquire()
            if request_id in log_query_responses:
                log_query_responses[request_id] = response_data
                # 通知等待的线程
                log_query_responses[request_id + '_event'] = True
                print(f"✅ [MQTT Response Handler] 响应已存储并通知等待线程: request_id={request_id}")
            else:
                print(f"⚠️  [MQTT Response Handler] 未找到对应的请求 ID: request_id={request_id}")
            log_query_lock.release()
        except Exception as e:
            import traceback
            print(f"❌ [MQTT Response Handler] 处理响应消息时出错: {e}")
            print(f"❌ [MQTT Response Handler] 错误堆栈: {traceback.format_exc()}")
    
    def handle_request(self, action, request_data):
        """处理 MQTT 请求，返回响应数据"""
        if action == 'user':
            return {
                "msg": "ok",
                "data": USERS
            }
        elif action == 'config':
            return {
                "msg": "ok",
                "data": NODE_INFO
            }
        elif action == 'outbound':
            return {
                "msg": "ok",
                "data": {
                    "outbounds": OUTBOUND_CONFIGS,
                    "user_mapping": USER_OUTBOUND_MAPPING
                }
            }
        elif action == 'routing':
            # 处理路由配置请求
            print(f"📋 [路由配置请求] 返回路由配置:")
            print(f"   配置内容: {json.dumps(ROUTING_CONFIG, indent=2, ensure_ascii=False)}")
            return {
                "msg": "ok",
                "data": ROUTING_CONFIG
            }
        elif action == 'submit':
            # 处理流量上报
            data = request_data.get('data', [])
            print(f"📈 [流量上报] 收到 {len(data)} 条流量记录:")
            for item in data:
                uid = item.get('uid', 'N/A')
                upload = item.get('u', 0)
                download = item.get('d', 0)
                print(f"   - UID {uid}: 上传 {upload} bytes, 下载 {download} bytes (总计: {upload + download} bytes)")
            return {"msg": "ok"}
        elif action == 'nodestatus':
            # 处理节点状态上报
            data = request_data.get('data', {})
            cpu = data.get('cpu', 'N/A')
            mem = data.get('mem', 'N/A')
            disk = data.get('disk', 'N/A')
            uptime = data.get('uptime', 'N/A')
            print(f"💻 [节点状态上报] CPU: {cpu}, 内存: {mem}, 磁盘: {disk}, 运行时间: {uptime} 秒")
            return {"msg": "ok"}
        elif action == 'onlineusers':
            # 处理在线用户上报
            data = request_data.get('data', [])
            print(f"👥 [在线用户上报] 收到 {len(data)} 个在线用户:")
            for item in data:
                uid = item.get('uid', 'N/A')
                ip = item.get('ip', 'N/A')
                print(f"   - UID {uid}, IP: {ip}")
            return {"msg": "ok"}
        elif action == 'illegal':
            # 处理非法行为上报
            data = request_data.get('data', [])
            print(f"⚠️  [非法行为上报] 收到 {len(data)} 条非法行为记录:")
            for item in data:
                uid = item.get('uid', 'N/A')
                print(f"   - UID {uid} 触发了非法行为")
            return {"msg": "ok"}
        elif action == 'outbound_failure':
            # 处理 outbound 连接失败上报
            data = request_data.get('data', {})
            outbound_tag = data.get('outbound_tag', 'N/A')
            error_msg = data.get('error', 'N/A')
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'
            
            # 从上报消息中获取 outbound 配置（节点上报的配置）
            outbound_config = data.get('config', {})
            
            # 显眼的故障上报日志
            print("\n" + "=" * 80)
            print("🔴🔴🔴  OUTBOUND 连接故障上报  🔴🔴🔴")
            print("=" * 80)
            print(f"  ⚠️  Outbound Tag:  {outbound_tag}")
            print(f"  ❌ 错误信息:      {error_msg}")
            print(f"  🕐 故障时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            
            # 显示 outbound 配置信息（从节点上报的配置中获取）
            if outbound_config:
                protocol = outbound_config.get('protocol', 'N/A')
                print(f"  📋 协议类型:      {protocol}")
                
                # 根据协议类型显示不同的配置信息
                settings = outbound_config.get('settings', {})
                if protocol == 'shadowsocks':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        method = server.get('method', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔐 加密方法:      {method}")
                elif protocol == 'vmess' or protocol == 'vless':
                    vnext = settings.get('vnext', [])
                    if vnext and len(vnext) > 0:
                        server_info = vnext[0]
                        address = server_info.get('address', 'N/A')
                        port = server_info.get('port', 'N/A')
                        users = server_info.get('users', [])
                        if users and len(users) > 0:
                            user_id = users[0].get('id', 'N/A')
                            print(f"  🌐 服务器地址:    {address}:{port}")
                            print(f"  👤 用户 ID:       {user_id[:8]}..." if len(user_id) > 8 else f"  👤 用户 ID:       {user_id}")
                elif protocol == 'trojan':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        password = server.get('password', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔑 密码:          {password[:8]}..." if len(password) > 8 else f"  🔑 密码:          {password}")
                else:
                    # 其他协议，显示完整配置（隐藏敏感信息）
                    config_str = json.dumps(outbound_config, ensure_ascii=False, indent=2)
                    print(f"  📄 配置详情:\n{config_str}")
            else:
                print(f"  ⚠️  未包含 outbound 配置信息")
            
            print("=" * 80 + "\n")
            
            # 可以在这里记录到数据库或触发告警
            return {"msg": "ok"}
        elif action == 'outbound_recovery':
            # 处理 outbound 连接恢复上报
            data = request_data.get('data', {})
            outbound_tag = data.get('outbound_tag', 'N/A')
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'
            
            # 从上报消息中获取 outbound 配置（节点上报的配置）
            outbound_config = data.get('config', {})
            
            # 显眼的恢复上报日志
            print("\n" + "=" * 80)
            print("🟢🟢🟢  OUTBOUND 连接恢复上报  🟢🟢🟢")
            print("=" * 80)
            print(f"  ✅ Outbound Tag:  {outbound_tag}")
            print(f"  🕐 恢复时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            
            # 显示 outbound 配置信息（从节点上报的配置中获取）
            if outbound_config:
                protocol = outbound_config.get('protocol', 'N/A')
                print(f"  📋 协议类型:      {protocol}")
                
                # 根据协议类型显示不同的配置信息
                settings = outbound_config.get('settings', {})
                if protocol == 'shadowsocks':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        method = server.get('method', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔐 加密方法:      {method}")
                elif protocol == 'vmess' or protocol == 'vless':
                    vnext = settings.get('vnext', [])
                    if vnext and len(vnext) > 0:
                        server_info = vnext[0]
                        address = server_info.get('address', 'N/A')
                        port = server_info.get('port', 'N/A')
                        users = server_info.get('users', [])
                        if users and len(users) > 0:
                            user_id = users[0].get('id', 'N/A')
                            print(f"  🌐 服务器地址:    {address}:{port}")
                            print(f"  👤 用户 ID:       {user_id[:8]}..." if len(user_id) > 8 else f"  👤 用户 ID:       {user_id}")
                elif protocol == 'trojan':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        password = server.get('password', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔑 密码:          {password[:8]}..." if len(password) > 8 else f"  🔑 密码:          {password}")
                else:
                    # 其他协议，显示完整配置（隐藏敏感信息）
                    config_str = json.dumps(outbound_config, ensure_ascii=False, indent=2)
                    print(f"  📄 配置详情:\n{config_str}")
            else:
                print(f"  ⚠️  未包含 outbound 配置信息")
            
            print("=" * 80 + "\n")
            
            # 可以在这里记录到数据库或更新状态
            return {"msg": "ok"}
        elif action == 'outbound_latency':
            # 处理单个 outbound 延迟上报（兼容旧版本）
            data = request_data.get('data', {})
            outbound_tag = data.get('outbound_tag', 'N/A')
            latency_ms = data.get('latency_ms', 0)
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'
            
            # 从上报消息中获取 outbound 配置（节点上报的配置）
            outbound_config = data.get('config', {})
            
            # 显眼的延迟上报日志
            latency_str = f"{latency_ms:.2f} ms"
            if latency_ms < 50:
                latency_emoji = "🟢"
            elif latency_ms < 100:
                latency_emoji = "🟡"
            elif latency_ms < 200:
                latency_emoji = "🟠"
            else:
                latency_emoji = "🔴"
            
            print("\n" + "=" * 80)
            print(f"{latency_emoji}  OUTBOUND 延迟上报  {latency_emoji}")
            print("=" * 80)
            print(f"  📍 Outbound Tag:  {outbound_tag}")
            print(f"  ⏱️  延迟:          {latency_str}")
            print(f"  🕐 上报时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            
            # 显示 outbound 配置信息（从节点上报的配置中获取）
            if outbound_config:
                protocol = outbound_config.get('protocol', 'N/A')
                print(f"  📋 协议类型:      {protocol}")
                
                # 根据协议类型显示不同的配置信息
                settings = outbound_config.get('settings', {})
                if protocol == 'shadowsocks':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        method = server.get('method', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔐 加密方法:      {method}")
                elif protocol == 'vmess' or protocol == 'vless':
                    vnext = settings.get('vnext', [])
                    if vnext and len(vnext) > 0:
                        server_info = vnext[0]
                        address = server_info.get('address', 'N/A')
                        port = server_info.get('port', 'N/A')
                        users = server_info.get('users', [])
                        if users and len(users) > 0:
                            user_id = users[0].get('id', 'N/A')
                            print(f"  🌐 服务器地址:    {address}:{port}")
                            print(f"  👤 用户 ID:       {user_id[:8]}..." if len(user_id) > 8 else f"  👤 用户 ID:       {user_id}")
                elif protocol == 'trojan':
                    servers = settings.get('servers', [])
                    if servers and len(servers) > 0:
                        server = servers[0]
                        address = server.get('address', 'N/A')
                        port = server.get('port', 'N/A')
                        password = server.get('password', 'N/A')
                        print(f"  🌐 服务器地址:    {address}:{port}")
                        print(f"  🔑 密码:          {password[:8]}..." if len(password) > 8 else f"  🔑 密码:          {password}")
                else:
                    # 其他协议，显示完整配置（隐藏敏感信息）
                    config_str = json.dumps(outbound_config, ensure_ascii=False, indent=2)
                    print(f"  📄 配置详情:\n{config_str}")
            else:
                print(f"  ⚠️  未包含 outbound 配置信息")
            
            print("=" * 80 + "\n")
            
            # 可以在这里记录到数据库或用于监控告警
            return {"msg": "ok"}
        elif action == 'outbound_latencies':
            # 处理批量 outbound 延迟上报
            data = request_data.get('data', {})
            latencies = data.get('latencies', [])
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'
            
            if not latencies or len(latencies) == 0:
                return {"msg": "ok"}
            
            # 显眼的批量延迟上报日志
            print("\n" + "=" * 80)
            print("📊📊📊  OUTBOUND 批量延迟上报  📊📊📊")
            print("=" * 80)
            print(f"  📦 上报数量:      {len(latencies)} 个 outbound")
            print(f"  🕐 上报时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            print("")
            
            # 按延迟值排序显示
            sorted_latencies = sorted(latencies, key=lambda x: x.get('latency_ms', 0))
            
            for i, latency_data in enumerate(sorted_latencies, 1):
                outbound_tag = latency_data.get('outbound_tag', 'N/A')
                latency_ms = latency_data.get('latency_ms', 0)
                
                # 根据延迟值选择颜色
                if latency_ms < 50:
                    latency_emoji = "🟢"
                elif latency_ms < 100:
                    latency_emoji = "🟡"
                elif latency_ms < 200:
                    latency_emoji = "🟠"
                else:
                    latency_emoji = "🔴"
                
                latency_str = f"{latency_ms:.2f} ms"
                print(f"  {i}. {latency_emoji} {outbound_tag:20s} - {latency_str:>10s}")
            
            print("=" * 80 + "\n")
            
            # 可以在这里记录到数据库或用于监控告警
            return {"msg": "ok"}
        elif action == 'outbound_udp_latency':
            # 单个 outbound UDP 延迟上报
            data = request_data.get('data', {})
            outbound_tag = data.get('outbound_tag', 'N/A')
            latency_ms = data.get('latency_ms', 0)
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'

            latency_str = f"{latency_ms:.2f} ms"
            if latency_ms < 50:
                latency_emoji = "🟢"
            elif latency_ms < 100:
                latency_emoji = "🟡"
            elif latency_ms < 200:
                latency_emoji = "🟠"
            else:
                latency_emoji = "🔴"

            print("\n" + "=" * 80)
            print(f"{latency_emoji}  UDP OUTBOUND 延迟上报  {latency_emoji}")
            print("=" * 80)
            print(f"  📍 Outbound Tag:  {outbound_tag}")
            print(f"  ⏱️  延迟:          {latency_str}")
            print(f"  🕐 上报时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            print("=" * 80 + "\n")
            return {"msg": "ok"}
        elif action == 'outbound_udp_latencies':
            # 批量 outbound UDP 延迟上报
            data = request_data.get('data', {})
            latencies = data.get('latencies', [])
            timestamp = data.get('timestamp', 0)
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp)) if timestamp > 0 else 'N/A'

            if not latencies or len(latencies) == 0:
                return {"msg": "ok"}

            print("\n" + "=" * 80)
            print("📊📊📊  UDP OUTBOUND 批量延迟上报  📊📊📊")
            print("=" * 80)
            print(f"  📦 上报数量:      {len(latencies)} 个 outbound")
            print(f"  🕐 上报时间:      {time_str}")
            print(f"  📡 节点 ID:       {request_data.get('node_id', 'N/A')}")
            print("")

            sorted_latencies = sorted(latencies, key=lambda x: x.get('latency_ms', 0))
            for i, latency_data in enumerate(sorted_latencies, 1):
                outbound_tag = latency_data.get('outbound_tag', 'N/A')
                latency_ms = latency_data.get('latency_ms', 0)
                if latency_ms < 50:
                    latency_emoji = "🟢"
                elif latency_ms < 100:
                    latency_emoji = "🟡"
                elif latency_ms < 200:
                    latency_emoji = "🟠"
                else:
                    latency_emoji = "🔴"
                latency_str = f"{latency_ms:.2f} ms"
                print(f"  {i}. {latency_emoji} {outbound_tag:20s} - {latency_str:>10s}")

            print("=" * 80 + "\n")
            return {"msg": "ok"}
        elif action == 'query_logs':
            # 日志查询请求应该由节点处理，这里不应该收到
            # 如果收到，说明是节点返回的响应（通过 handle_response 处理）
            return {"msg": "error", "error": "query_logs 应该由节点处理"}
        elif action == 'udp_probe':
            # UDP 探测请求应该由节点处理，管理端不应该处理
            # 如果收到，说明是管理端自己发送的请求，应该忽略（让节点端处理）
            print(f"⚠️  [MQTT] 收到 udp_probe 请求，应该由节点端处理，管理端忽略")
            return None  # 返回 None 表示不发送响应，让节点端处理
        else:
            return {"msg": "error", "error": f"未知的 action: {action}"}
    
    def start(self):
        try:
            self.client.connect(self.broker_host, self.broker_port, 60)
            self.client.loop_start()
        except Exception as e:
            print(f"❌ MQTT 启动失败: {e}")
    
    def stop(self):
        self.client.loop_stop()
        self.client.disconnect()
    
    def publish_update_notification(self, node_id, update_type):
        """
        推送配置更新通知到节点
        update_type: 'user', 'outbound', 'config'
        """
        # 检查连接状态（兼容不同版本的 paho-mqtt）
        try:
            # paho-mqtt >= 1.6.0
            if hasattr(self.client, 'is_connected') and not self.client.is_connected():
                print(f"⚠️  MQTT 未连接，无法推送更新通知")
                return False
        except:
            pass
        
        topic = f"xrayr/node/{node_id}/update/{update_type}"
        message = {
            "type": update_type,
            "timestamp": time.time(),
            "action": "update"
        }
        
        try:
            result = self.client.publish(topic, json.dumps(message), qos=1)
            if result is not None and result.rc == mqtt.MQTT_ERR_SUCCESS:
                print(f"📢 [MQTT] 已推送更新通知: {topic} (type={update_type})")
                return True
            else:
                rc = result.rc if result else "unknown"
                print(f"❌ [MQTT] 推送更新通知失败: {topic}, rc={rc}")
                return False
        except Exception as e:
            print(f"❌ [MQTT] 推送更新通知时出错: {e}")
            return False
    
    def query_node_logs(self, node_id, query_params, timeout=10):
        """
        通过 MQTT 查询节点日志
        query_params: 包含 uid, days, event, limit 等参数
        返回日志查询结果
        """
        if not MQTT_AVAILABLE:
            return None
        
        # 生成请求 ID
        request_id = f"log_query_{int(time.time() * 1000)}_{node_id}"
        
        # 准备请求数据
        request_data = {
            "node_id": node_id,
            "token": "123",  # 使用配置的 token
            "action": "query_logs",
            "data": query_params
        }
        
        # 初始化响应等待
        log_query_lock.acquire()
        log_query_responses[request_id] = None
        log_query_responses[request_id + '_event'] = False
        log_query_lock.release()
        
        try:
            # 发布请求
            request_topic = f"xrayr/node/{node_id}/request/{request_id}"
            request_json = json.dumps(request_data)
            result = self.client.publish(request_topic, request_json, qos=1)
            
            if result is None or result.rc != mqtt.MQTT_ERR_SUCCESS:
                print(f"❌ [MQTT] 发布日志查询请求失败")
                return None
            
            # 等待响应（最多等待 timeout 秒）
            start_time = time.time()
            while time.time() - start_time < timeout:
                log_query_lock.acquire()
                if log_query_responses.get(request_id) is not None:
                    response = log_query_responses[request_id]
                    # 清理
                    del log_query_responses[request_id]
                    if request_id + '_event' in log_query_responses:
                        del log_query_responses[request_id + '_event']
                    log_query_lock.release()
                    return response
                log_query_lock.release()
                time.sleep(0.1)  # 等待 100ms
            
            # 超时
            log_query_lock.acquire()
            if request_id in log_query_responses:
                del log_query_responses[request_id]
            if request_id + '_event' in log_query_responses:
                del log_query_responses[request_id + '_event']
            log_query_lock.release()
            print(f"⚠️  [MQTT] 日志查询超时")
            return None
            
        except Exception as e:
            print(f"❌ [MQTT] 查询日志时出错: {e}")
            log_query_lock.acquire()
            if request_id in log_query_responses:
                del log_query_responses[request_id]
            if request_id + '_event' in log_query_responses:
                del log_query_responses[request_id + '_event']
            log_query_lock.release()
            return None

    def query_udp_latency(self, node_id, outbound_tag, timeout=UDP_PROBE_TIMEOUT):
        """
        主动查询指定 outbound 的 UDP 延迟
        需要节点支持 udp_probe 请求
        """
        print(f"🔍 [UDP Probe API] 开始查询 UDP 延迟: node_id={node_id}, outbound_tag={outbound_tag}, timeout={timeout}")
        
        if not MQTT_AVAILABLE:
            print(f"❌ [UDP Probe API] MQTT 不可用，无法查询 UDP 延迟")
            return None

        request_id = f"udp_probe_{int(time.time() * 1000)}_{node_id}_{outbound_tag}"
        request_data = {
            "node_id": node_id,
            "token": "123",
            "action": "udp_probe",
            "data": {
                "outbound_tag": outbound_tag
            }
        }

        print(f"📤 [UDP Probe API] 准备发送 MQTT 请求: request_id={request_id}, data={json.dumps(request_data)}")

        log_query_lock.acquire()
        log_query_responses[request_id] = None
        log_query_responses[request_id + '_event'] = False
        log_query_lock.release()

        try:
            request_topic = f"xrayr/node/{node_id}/request/{request_id}"
            request_json = json.dumps(request_data)
            print(f"📡 [UDP Probe API] 发布 MQTT 消息到主题: {request_topic}")
            result = self.client.publish(request_topic, request_json, qos=1)

            if result is None or result.rc != mqtt.MQTT_ERR_SUCCESS:
                print(f"❌ [UDP Probe API] 发布 UDP 探测请求失败: rc={result.rc if result else 'None'}")
                return None

            print(f"✅ [UDP Probe API] MQTT 消息发布成功，等待响应 (timeout={timeout}s)...")
            start_time = time.time()
            while time.time() - start_time < timeout:
                log_query_lock.acquire()
                if log_query_responses.get(request_id) is not None:
                    response = log_query_responses[request_id]
                    elapsed = time.time() - start_time
                    print(f"✅ [UDP Probe API] 收到响应 (耗时 {elapsed:.2f}s): {json.dumps(response)}")
                    del log_query_responses[request_id]
                    if request_id + '_event' in log_query_responses:
                        del log_query_responses[request_id + '_event']
                    log_query_lock.release()
                    return response
                log_query_lock.release()
                time.sleep(0.1)

            log_query_lock.acquire()
            if request_id in log_query_responses:
                del log_query_responses[request_id]
            if request_id + '_event' in log_query_responses:
                del log_query_responses[request_id + '_event']
            log_query_lock.release()
            elapsed = time.time() - start_time
            print(f"⚠️  [UDP Probe API] UDP 探测超时 (等待了 {elapsed:.2f}s)，未收到响应")
            return None

        except Exception as e:
            import traceback
            print(f"❌ [UDP Probe API] 查询 UDP 延迟时出错: {e}")
            print(f"❌ [UDP Probe API] 错误堆栈: {traceback.format_exc()}")
            log_query_lock.acquire()
            if request_id in log_query_responses:
                del log_query_responses[request_id]
            if request_id + '_event' in log_query_responses:
                del log_query_responses[request_id + '_event']
            log_query_lock.release()
            return None


def _notify_update(update_type):
    """通知配置更新（通过 MQTT 推送）"""
    global mqtt_handler_global
    if mqtt_handler_global:
        node_id = NODE_INFO.get("node_id", 41)
        mqtt_handler_global.publish_update_notification(node_id, update_type)
    else:
        print(f"⚠️  MQTT 处理器未初始化，无法推送更新通知 (type={update_type})")


def udp_probe_scheduler():
    """定时主动触发 UDP 延迟查询"""
    while True:
        if UDP_PROBE_ENABLED and mqtt_handler_global:
            node_id = NODE_INFO.get("node_id", 41)
            with config_lock:
                targets = [o.get("tag") for o in OUTBOUND_CONFIGS if o.get("tag")]
            for tag in targets:
                result = mqtt_handler_global.query_udp_latency(node_id, tag, timeout=UDP_PROBE_TIMEOUT)
                if result:
                    udp_probe_results[tag] = {
                        "latency_ms": result.get("latency_ms", None),
                        "timestamp": int(time.time())
                    }
        time.sleep(UDP_PROBE_INTERVAL)


if __name__ == '__main__':
    # 启动 HTTP 服务器
    http_server = HTTPServer(('127.0.0.1', 667), APIHandler)
    http_thread = threading.Thread(target=http_server.serve_forever, daemon=True)
    http_thread.start()
    
    # 启动 MQTT 服务器（如果可用）
    mqtt_handler = None
    if MQTT_AVAILABLE:
        try:
            # MQTT 配置
            MQTT_BROKER_HOST = '127.0.0.1'
            MQTT_BROKER_PORT = 1883
            MQTT_USERNAME = None  # 可选
            MQTT_PASSWORD = None  # 可选
            
            mqtt_handler = MQTTHandler(
                broker_host=MQTT_BROKER_HOST,
                broker_port=MQTT_BROKER_PORT,
                username=MQTT_USERNAME,
                password=MQTT_PASSWORD
            )
            mqtt_handler.start()
            # 设置全局引用
            mqtt_handler_global = mqtt_handler
        except Exception as e:
            print(f"⚠️  MQTT 服务器启动失败: {e}")
            mqtt_handler = None
            mqtt_handler_global = None

    # 启动 UDP 探测定时器
    probe_thread = threading.Thread(target=udp_probe_scheduler, daemon=True)
    probe_thread.start()
    
    print("=" * 70)
    print("🚀 本地 API 服务器已启动")
    print("=" * 70)
    print(f"📋 用户数量: {len(USERS)}")
    print(f"🌐 上游代理数量: {len(OUTBOUND_CONFIGS)}")
    print("")
    print("📡 HTTP API 接口 (http://127.0.0.1:667):")
    print("  - GET ?act=user          - 获取用户列表")
    print("  - GET ?act=config        - 获取节点配置")
    print("  - GET ?act=outbound      - 获取上游代理配置")
    print("")
    print("🔧 HTTP 管理接口 (http://127.0.0.1:667):")
    print("  【用户管理】")
    print("  - POST   /api/admin/user       - 添加用户")
    print("  - PUT    /api/admin/user/{id}  - 更新用户")
    print("  - DELETE /api/admin/user/{id}  - 删除用户")
    print("  【上游代理管理】")
    print("  - POST   /api/admin/outbound      - 添加上游代理")
    print("  - PUT    /api/admin/outbound/{tag} - 更新上游代理")
    print("  - DELETE /api/admin/outbound/{tag} - 删除上游代理")
    print("  【用户映射管理】")
    print("  - POST   /api/admin/mapping      - 添加用户映射")
    print("  - DELETE /api/admin/mapping/{uuid} - 删除用户映射")
    print("  【路由配置管理】")
    print("  - POST   /api/admin/routing      - 更新整个路由配置")
    print("  - POST   /api/admin/routing/rule  - 添加路由规则")
    print("  - PUT    /api/admin/routing/rule/{index} - 更新路由规则")
    print("  - DELETE /api/admin/routing/rule/{index} - 删除路由规则")
    print("")
    if mqtt_handler:
        print("📡 MQTT API 接口:")
        print(f"  - Broker: {MQTT_BROKER_HOST}:{MQTT_BROKER_PORT}")
        print("  - 请求主题: xrayr/node/{node_id}/request/{request_id}")
        print("  - 更新通知主题: xrayr/node/{node_id}/update/{type}")
        print("  - 支持的操作: user, config, outbound, routing, submit, nodestatus, onlineusers, illegal, outbound_failure, outbound_recovery, outbound_latency, outbound_latencies")
        print("  - 更新通知类型: user, outbound, config, routing")
        print("  - 上报类型: submit (流量), nodestatus (节点状态), onlineusers (在线用户), illegal (非法行为), outbound_failure (连接失败), outbound_recovery (连接恢复), outbound_latency (单个延迟), outbound_latencies (批量延迟)")
        print("")
    print("✨ 热重载支持:")
    print("  - 通过 HTTP 管理接口修改配置后，立即通过 MQTT 推送更新通知")
    print("  - 节点收到通知后会立即重新拉取配置，无需等待定时周期")
    print("  - 如果 MQTT 不可用，修改配置后 XrayR 会在下次更新周期（默认 60 秒）自动重载")
    print("=" * 70)
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\n⏹️  正在停止服务器...")
        http_server.shutdown()
        if mqtt_handler:
            mqtt_handler.stop()
        print("✅ 服务器已停止")

