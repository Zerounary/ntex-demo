# MQTT TLS 证书说明

本目录包含用于 MQTT Broker TLS 加密的证书文件。

## 证书文件

- **ca.cert.pem** - CA（证书颁发机构）证书
- **ca.key** - CA 私钥（请妥善保管，不要泄露）
- **server.cert.pem** - 服务器证书
- **server.key** - 服务器私钥（用于 MQTT Broker）
- **client.cert.pem** - 客户端证书
- **client.key** - 客户端私钥（用于 MQTT 客户端连接）

## 配置说明

证书已在 `config.toml` 中配置：

```toml
[v4.v4-1.tls]
certpath = "certs/server.cert.pem"
keypath = "certs/server.key"
capath = "certs/ca.cert.pem"
```

## 客户端连接

使用 TLS 连接 MQTT Broker 时，客户端需要：

1. **服务器证书验证**：使用 `ca.cert.pem` 验证服务器证书
2. **客户端证书认证**（可选）：使用 `client.cert.pem` 和 `client.key` 进行客户端证书认证

## 重要提示

⚠️ **这些是自签名证书，仅用于开发和测试环境！**

生产环境请使用：
- 由受信任的 CA 颁发的证书（如 Let's Encrypt）
- 或者使用企业内部的 CA 颁发的证书

## 重新生成证书

如果需要重新生成证书，运行：

```powershell
powershell -ExecutionPolicy Bypass -File generate_certs.ps1
```
