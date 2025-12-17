# MQTT TLS 连接测试脚本

## 简介

`test_mqtt_tls.py` 是一个用于测试 MQTT Broker 是否支持普通连接和 TLS 连接的 Python3 脚本。

## 安装依赖

```bash
pip install paho-mqtt
# 或
pip3 install paho-mqtt
```

## 使用方法

### 基本用法（测试默认配置）

```bash
python3 test_mqtt_tls.py
```

### 指定主机和端口

```bash
python3 test_mqtt_tls.py --host 127.0.0.1 --port 1883
```

### 使用 CA 证书测试 TLS

```bash
python3 test_mqtt_tls.py --ca-cert certs/ca.cert.pem
```

### 使用完整证书链测试 TLS（包含客户端证书认证）

```bash
python3 test_mqtt_tls.py \
    --ca-cert certs/ca.cert.pem \
    --client-cert certs/client.cert.pem \
    --client-key certs/client.key
```

## 自动发现证书

脚本会自动在以下位置查找证书文件（如果未通过参数指定）：

- `certs/ca.cert.pem`
- `XrayR/certs/ca.cert.pem`
- `../certs/ca.cert.pem`

## 测试结果说明

脚本会测试两种连接方式：

1. **普通 MQTT 连接** (tcp://host:port)
   - 不使用 TLS 加密
   - 用于验证 Broker 是否接受非加密连接

2. **TLS MQTT 连接** (ssl://host:port)
   - 使用 TLS/SSL 加密
   - 可以验证服务器证书
   - 可以配置客户端证书认证

## 结果分析

### ✅ 最佳情况
- 普通连接：失败 ❌
- TLS 连接：成功 ✅
- **说明**：Broker 已正确配置为仅接受 TLS 连接，安全性良好

### ⚠️ 警告情况 1
- 普通连接：成功 ✅
- TLS 连接：成功 ✅
- **说明**：Broker 同时支持两种连接方式，建议禁用普通连接

### ⚠️ 警告情况 2
- 普通连接：成功 ✅
- TLS 连接：失败 ❌
- **说明**：Broker 可能未启用 TLS 或 TLS 配置有误

### ❌ 错误情况
- 普通连接：失败 ❌
- TLS 连接：失败 ❌
- **说明**：Broker 可能未运行、端口不正确或网络问题

## 验证 TLS 是否真正使用

### 方法 1: 查看脚本输出
如果 TLS 连接成功，说明通信已加密。

### 方法 2: 使用 Wireshark 抓包
1. 启动 Wireshark
2. 抓取 `127.0.0.1:1883` 的流量
3. 如果看到 TLS 握手（Client Hello, Server Hello 等），说明使用了 TLS
4. 如果看到明文 MQTT 数据包，说明未使用 TLS

### 方法 3: 查看 Broker 日志
检查 Broker 启动日志，应该看到 TLS 相关的配置信息。

### 方法 4: 查看客户端日志
- XrayR: 查看日志中是否有 "🔒 [MQTT] TLS 已启用"
- Rust 客户端: 查看日志中是否有 "🔒 [MQTT] 配置 TLS 连接"

## 故障排查

### TLS 连接失败

1. **检查证书路径**
   ```bash
   ls -la certs/ca.cert.pem
   ls -la certs/client.cert.pem
   ls -la certs/client.key
   ```

2. **检查证书格式**
   ```bash
   openssl x509 -in certs/ca.cert.pem -text -noout
   ```

3. **检查 Broker 配置**
   - 确认 `config.toml` 中 `[v4.v4-1.tls]` 部分已配置
   - 确认证书路径正确

4. **检查端口**
   - 确认 Broker 监听的端口
   - TLS 可以在 1883 端口使用（如果配置了 TLS）

### 普通连接成功但应该失败

如果希望禁用普通连接，只允许 TLS：

1. 检查 Broker 配置
2. 确认是否有两个监听器（一个普通，一个 TLS）
3. 如果只有一个监听器且配置了 TLS，普通连接应该失败

## 示例输出

```
============================================================
MQTT 连接测试工具
============================================================
目标服务器: 127.0.0.1:1883
CA 证书: certs/ca.cert.pem
客户端证书: certs/client.cert.pem
客户端密钥: certs/client.key

[测试 1] 普通 MQTT 连接 (tcp://127.0.0.1:1883)
------------------------------------------------------------
  正在连接到 127.0.0.1:1883...
  ❌ 普通 MQTT 连接失败: 连接失败（on_connect_fail）

[测试 2] TLS MQTT 连接 (ssl://127.0.0.1:1883)
------------------------------------------------------------
  ✓ 已加载 CA 证书: certs/ca.cert.pem
  ✓ 已加载客户端证书: certs/client.cert.pem
  正在使用 TLS 连接到 127.0.0.1:1883...
  ✅ TLS MQTT 连接成功

============================================================
测试结果总结
============================================================
普通 MQTT 连接: ❌ 失败
  原因: 连接失败（on_connect_fail）
TLS MQTT 连接:  ✅ 成功
  原因: TLS 连接成功

============================================================
分析结果
============================================================
✅ 最佳情况: 仅 TLS 连接成功
   这意味着 Broker 已正确配置为仅接受 TLS 连接
   普通连接被拒绝，安全性良好
```

