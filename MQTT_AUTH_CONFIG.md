# MQTT 认证配置说明

## 概述

rumqttd 支持用户名密码认证。已在 `config.toml` 中配置了两个账号：

- **manage** - 管理端账号
- **node** - 节点端账号

## 配置位置

在 `config.toml` 的 `[v4.v4-1.connections]` 部分：

```toml
[v4.v4-1.connections]
connection_timeout_ms = 60000
max_payload_size = 104857600
max_inflight_count = 100
dynamic_filters = true
# 用户名密码认证配置
auth = { manage = "manage_password_123", node = "node_password_123" }
```

## 账号信息

### 管理端账号（manage）
- **用户名**: `manage`
- **密码**: `manage_password_123`
- **用途**: 用于管理端（`src/infrastructure/mqtt_client.rs`）连接 MQTT Broker

### 节点端账号（node）
- **用户名**: `node`
- **密码**: `node_password_123`
- **用途**: 用于节点端（XrayR）连接 MQTT Broker

## 客户端配置

### Rust 客户端（管理端）

代码已自动配置用户名密码认证。默认使用 `manage` 账号，可通过环境变量修改：

```powershell
# 使用默认账号（manage）
cargo run

# 或通过环境变量指定
$env:MQTT_USERNAME='manage'
$env:MQTT_PASSWORD='manage_password_123'
cargo run
```

代码实现（`src/infrastructure/mqtt_client.rs`）：
```rust
// 设置用户名和密码认证（从环境变量读取，默认使用 manage 账号）
let mqtt_username = env::var("MQTT_USERNAME")
    .unwrap_or_else(|_| "manage".to_string());
let mqtt_password = env::var("MQTT_PASSWORD")
    .unwrap_or_else(|_| "manage_password_123".to_string());
mqttoptions.set_credentials(&mqtt_username, &mqtt_password);
```

### XrayR 客户端（节点端）

在 `XrayR/config.yml` 中，需要在 `ApiHost` 中包含用户名和密码：

```yaml
ApiHost: "mqtt://node:node_password_123@192.168.1.136:1883"
```

或者使用环境变量（如果 XrayR 支持）：

```powershell
$env:MQTT_USERNAME='node'
$env:MQTT_PASSWORD='node_password_123'
```

**注意**：XrayR 的 MQTT 客户端代码（`XrayR/api/mqttpanel/mqttpanel.go`）可能需要修改以支持从 URL 中解析用户名和密码，或者从环境变量读取。

## 修改密码

如需修改密码，直接编辑 `config.toml` 中的 `auth` 配置：

```toml
auth = { manage = "新密码", node = "新密码" }
```

修改后需要重启 MQTT Broker 才能生效。

## 添加更多账号

可以在 `auth` 配置中添加更多账号：

```toml
auth = { 
    manage = "manage_password_123", 
    node = "node_password_123",
    user3 = "password3",
    user4 = "password4"
}
```

## 禁用认证

如需禁用认证，可以注释掉或删除 `auth` 配置行：

```toml
# auth = { manage = "manage_password_123", node = "node_password_123" }
```

## 安全建议

1. **生产环境**：请使用强密码，建议使用随机生成的密码
2. **密码管理**：不要将密码提交到版本控制系统
3. **定期更换**：定期更换密码以提高安全性
4. **结合 TLS**：建议同时启用 TLS 加密，即使密码泄露，通信仍然是加密的

## 相关文件

- `config.toml` - MQTT Broker 配置文件
- `src/infrastructure/mqtt_client.rs` - Rust MQTT 客户端
- `XrayR/config.yml` - XrayR 配置文件
- `rumqttd/src/lib.rs` - rumqttd 认证实现

## 参考

- rumqttd 认证文档：https://github.com/bytebeamio/rumqtt
- rumqttd 配置示例：`rumqttd/rumqttd.toml`

