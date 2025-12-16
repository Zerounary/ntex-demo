# MQTT Broker 使用 rumqttd

## 编译和运行

```bash
cd mqtt_broker
cargo run
```

## 配置说明

当前代码使用 `rumqttd` 的默认配置。如果需要自定义端口或其他配置，需要根据 `rumqttd 0.20.0` 的实际 API 进行调整。

### 查看 rumqttd API 文档

```bash
cd mqtt_broker
cargo doc --open --no-deps
```

这会打开 rumqttd 的 API 文档，你可以查看 `Config` 结构体的实际字段。

### 常见配置示例

根据 rumqttd 0.20.0 的实际 API，配置可能类似：

```rust
use rumqttd::{Broker, Config};

let config = Config {
    // 根据实际 API 调整这些字段
    // 可能需要设置 id, router, v4, v5, ws 等字段
    ..Default::default()
};
```

## 测试连接

启动 Broker 后，可以使用 `mosquitto` 客户端测试：

```bash
# 订阅主题
mosquitto_sub -h localhost -p 1883 -t "test/topic"

# 发布消息（在另一个终端）
mosquitto_pub -h localhost -p 1883 -t "test/topic" -m "Hello MQTT"
```

## 与 XrayR 集成

修改 XrayR 配置：

```yaml
Nodes:
  - PanelType: "MQTT"
    ApiConfig:
      ApiHost: "mqtt://localhost:1883"  # 使用本地 rumqttd
      ApiKey: "your-api-key"
      NodeID: 41
```

## 故障排查

如果启动失败，检查：

1. **端口是否被占用**
   ```bash
   # Windows
   netstat -ano | findstr :1883
   
   # Linux
   lsof -i :1883
   ```

2. **查看错误信息**
   - 代码会打印详细的错误信息
   - 根据错误信息调整配置

3. **查看 rumqttd 文档**
   ```bash
   cargo doc --open --no-deps
   ```

## 注意事项

- `rumqttd 0.20.0` 的 API 可能与早期版本不同
- 如果遇到编译错误，请查看 `Config` 结构体的实际字段
- 默认配置通常已经足够使用

