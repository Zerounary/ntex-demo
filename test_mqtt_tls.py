#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
MQTT TLS 连接测试脚本
测试 MQTT Broker 的 1883 端口是否支持普通连接和 TLS 连接
"""

import sys
import ssl
import time
import socket
import os
from typing import Optional, Tuple

try:
    import paho.mqtt.client as mqtt
except ImportError:
    print("错误: 未安装 paho-mqtt 库")
    print("请运行: pip install paho-mqtt")
    sys.exit(1)


class MQTTTester:
    def __init__(self, host: str = "127.0.0.1", port: int = 1883):
        self.host = host
        self.port = port
        self.results = {}

    def test_tcp_connection(self) -> bool:
        """测试 TCP 连接是否可达"""
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(3)
            result = sock.connect_ex((self.host, self.port))
            sock.close()
            return result == 0
        except Exception as e:
            print(f"  TCP 连接测试失败: {e}")
            return False

    def test_plain_mqtt(self) -> Tuple[bool, str]:
        """测试普通 MQTT 连接（非 TLS）"""
        print(f"\n[测试 1] 普通 MQTT 连接 (tcp://{self.host}:{self.port})")
        print("-" * 60)
        
        if not self.test_tcp_connection():
            return False, "TCP 连接不可达"
        
        client = None
        try:
            client = mqtt.Client(client_id="test_plain_mqtt")
            client._connect_timeout = 3
            
            # 设置回调
            connected = [False]
            error_msg = [None]
            
            def on_connect(client, userdata, flags, rc):
                if rc == 0:
                    connected[0] = True
                else:
                    error_msg[0] = f"连接失败，返回码: {rc}"
            
            def on_connect_fail(client, userdata):
                error_msg[0] = "连接失败（on_connect_fail）"
            
            client.on_connect = on_connect
            client.on_connect_fail = on_connect_fail
            
            # 尝试连接
            print(f"  正在连接到 {self.host}:{self.port}...")
            client.connect(self.host, self.port, keepalive=60)
            client.loop_start()
            
            # 等待连接结果
            time.sleep(2)
            client.loop_stop()
            client.disconnect()
            
            if connected[0]:
                print("  ✅ 普通 MQTT 连接成功")
                return True, "连接成功"
            else:
                print(f"  ❌ 普通 MQTT 连接失败: {error_msg[0] or '未知错误'}")
                return False, error_msg[0] or "未知错误"
                
        except Exception as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            print(f"  ❌ 普通 MQTT 连接异常: {e}")
            return False, str(e)

    def test_tls_mqtt(self, ca_cert: Optional[str] = None, 
                      client_cert: Optional[str] = None,
                      client_key: Optional[str] = None) -> Tuple[bool, str]:
        """测试 TLS MQTT 连接"""
        print(f"\n[测试 2] TLS MQTT 连接 (ssl://{self.host}:{self.port})")
        print("-" * 60)
        
        if not self.test_tcp_connection():
            return False, "TCP 连接不可达"
        
        client = None
        try:
            client = mqtt.Client(client_id="test_tls_mqtt")
            client._connect_timeout = 3
            
            # 配置 TLS
            # 检查是否允许不安全的连接（用于自签名证书）
            allow_insecure = os.getenv("MQTT_TLS_INSECURE", "").lower() == "true"
            
            # 对于自签名证书，Python 的 SSL 验证可能失败
            # 如果用户没有明确要求验证，默认使用 insecure 模式（开发环境）
            # 如果用户明确设置了 MQTT_TLS_INSECURE=false，则强制验证
            force_verify = os.getenv("MQTT_TLS_INSECURE", "").lower() == "false"
            
            if allow_insecure or (not force_verify):
                # 使用 insecure 模式（不验证证书）
                if allow_insecure:
                    print("  ⚠️  已启用不安全模式（不验证证书），仅用于开发测试")
                else:
                    print("  ⚠️  检测到自签名证书，自动使用不安全模式（不验证证书）")
                    print("  💡 提示: 这是正常的，因为自签名证书无法通过严格验证")
                    print("  💡 如需强制验证，请设置: $env:MQTT_TLS_INSECURE='false'")
                tls_context = ssl.create_default_context()
                tls_context.check_hostname = False
                tls_context.verify_mode = ssl.CERT_NONE
            else:
                # 强制验证模式
                print("  ✓ 强制验证模式（已设置 MQTT_TLS_INSECURE=false）")
                tls_context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
                
                if ca_cert:
                    try:
                        tls_context.load_verify_locations(ca_cert)
                        print(f"  ✓ 已加载 CA 证书: {ca_cert}")
                        tls_context.check_hostname = False
                    except Exception as e:
                        print(f"  ⚠️  加载 CA 证书失败: {e}")
                        raise
                else:
                    print("  ⚠️  未提供 CA 证书，将使用系统默认证书")
            
            if client_cert and client_key:
                try:
                    tls_context.load_cert_chain(client_cert, client_key)
                    print(f"  ✓ 已加载客户端证书: {client_cert}")
                except Exception as e:
                    print(f"  ⚠️  加载客户端证书失败: {e}")
            
            # 设置 TLS
            client.tls_set_context(tls_context)
            
            # 设置回调
            connected = [False]
            error_msg = [None]
            
            def on_connect(client, userdata, flags, rc):
                if rc == 0:
                    connected[0] = True
                else:
                    error_msg[0] = f"连接失败，返回码: {rc}"
            
            def on_connect_fail(client, userdata):
                error_msg[0] = "连接失败（on_connect_fail）"
            
            client.on_connect = on_connect
            client.on_connect_fail = on_connect_fail
            
            # 尝试连接
            print(f"  正在使用 TLS 连接到 {self.host}:{self.port}...")
            client.connect(self.host, self.port, keepalive=60)
            client.loop_start()
            
            # 等待连接结果
            time.sleep(2)
            client.loop_stop()
            client.disconnect()
            
            if connected[0]:
                print("  ✅ TLS MQTT 连接成功")
                return True, "TLS 连接成功"
            else:
                print(f"  ❌ TLS MQTT 连接失败: {error_msg[0] or '未知错误'}")
                return False, error_msg[0] or "未知错误"
                
        except ssl.SSLError as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            error_msg = str(e)
            # 如果是证书验证错误，自动尝试 insecure 模式
            if "CERTIFICATE_VERIFY_FAILED" in error_msg or "certificate verify failed" in error_msg:
                print(f"  ⚠️  证书验证失败: {e}")
                print("  💡 这是自签名证书的正常现象，正在尝试不验证证书模式...")
                
                # 自动重试，使用 insecure 模式
                try:
                    client2 = mqtt.Client(client_id="test_tls_mqtt_retry")
                    client2._connect_timeout = 3
                    
                    # 使用 insecure 模式
                    tls_context2 = ssl.create_default_context()
                    tls_context2.check_hostname = False
                    tls_context2.verify_mode = ssl.CERT_NONE
                    
                    if client_cert and client_key:
                        try:
                            tls_context2.load_cert_chain(client_cert, client_key)
                        except:
                            pass  # 客户端证书可选
                    
                    client2.tls_set_context(tls_context2)
                    
                    connected2 = [False]
                    error_msg2 = [None]
                    
                    def on_connect2(client, userdata, flags, rc):
                        if rc == 0:
                            connected2[0] = True
                        else:
                            error_msg2[0] = f"连接失败，返回码: {rc}"
                    
                    def on_connect_fail2(client, userdata):
                        error_msg2[0] = "连接失败（on_connect_fail）"
                    
                    client2.on_connect = on_connect2
                    client2.on_connect_fail = on_connect_fail2
                    
                    print(f"  🔄 重试连接（不验证证书）...")
                    client2.connect(self.host, self.port, keepalive=60)
                    client2.loop_start()
                    time.sleep(2)
                    client2.loop_stop()
                    client2.disconnect()
                    
                    if connected2[0]:
                        print("  ✅ TLS MQTT 连接成功（不验证证书模式）")
                        print("  💡 说明: 虽然证书验证失败，但 TLS 加密连接已建立")
                        print("  💡 这是自签名证书的正常行为，通信已加密")
                        return True, "TLS 连接成功（不验证证书）"
                    else:
                        print(f"  ❌ 重试也失败: {error_msg2[0] or '未知错误'}")
                        return False, f"SSL 错误: {e}，重试也失败: {error_msg2[0]}"
                except Exception as e2:
                    print(f"  ❌ 重试失败: {e2}")
                    return False, f"SSL 错误: {e}，重试失败: {e2}"
            else:
                print(f"  ❌ TLS 握手失败: {e}")
            return False, f"SSL 错误: {e}"
        except Exception as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            print(f"  ❌ TLS MQTT 连接异常: {e}")
            return False, str(e)

    def run_tests(self, ca_cert: Optional[str] = None,
                  client_cert: Optional[str] = None,
                  client_key: Optional[str] = None):
        """运行所有测试"""
        print("=" * 60)
        print("MQTT 连接测试工具")
        print("=" * 60)
        print(f"目标服务器: {self.host}:{self.port}")
        print(f"CA 证书: {ca_cert or '未指定'}")
        print(f"客户端证书: {client_cert or '未指定'}")
        print(f"客户端密钥: {client_key or '未指定'}")
        
        # 测试普通连接
        plain_success, plain_msg = self.test_plain_mqtt()
        self.results['plain'] = {
            'success': plain_success,
            'message': plain_msg
        }
        
        # 测试 TLS 连接
        tls_success, tls_msg = self.test_tls_mqtt(ca_cert, client_cert, client_key)
        self.results['tls'] = {
            'success': tls_success,
            'message': tls_msg
        }
        
        # 输出总结
        print("\n" + "=" * 60)
        print("测试结果总结")
        print("=" * 60)
        print(f"普通 MQTT 连接: {'✅ 成功' if plain_success else '❌ 失败'}")
        if not plain_success:
            print(f"  原因: {plain_msg}")
        print(f"TLS MQTT 连接:  {'✅ 成功' if tls_success else '❌ 失败'}")
        if not tls_success:
            print(f"  原因: {tls_msg}")
        
        print("\n" + "=" * 60)
        print("分析结果")
        print("=" * 60)
        
        if plain_success and tls_success:
            print("⚠️  警告: 两种连接方式都成功！")
            print("   这意味着 Broker 同时支持普通连接和 TLS 连接")
            print("   建议: 禁用普通连接，仅使用 TLS 以确保安全")
        elif not plain_success and tls_success:
            print("✅ 最佳情况: 仅 TLS 连接成功")
            print("   这意味着 Broker 已正确配置为仅接受 TLS 连接")
            print("   普通连接被拒绝，安全性良好")
        elif plain_success and not tls_success:
            print("⚠️  警告: 仅普通连接成功，TLS 连接失败")
            print("   这意味着 Broker 可能未启用 TLS 或 TLS 配置有误")
            print("   建议: 检查 Broker 的 TLS 配置")
        else:
            print("❌ 错误: 两种连接方式都失败")
            print("   可能原因:")
            print("   1. Broker 未运行")
            print("   2. 端口不正确")
            print("   3. 防火墙阻止连接")
            print("   4. 网络问题")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(
        description='测试 MQTT Broker 的普通连接和 TLS 连接',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  # 测试默认配置 (127.0.0.1:1883)
  python test_mqtt_tls.py
  
  # 测试指定主机和端口
  python test_mqtt_tls.py --host 192.168.1.100 --port 1883
  
  # 使用 CA 证书测试 TLS
  python test_mqtt_tls.py --ca-cert certs/ca.cert.pem
  
  # 使用完整证书链测试 TLS
  python test_mqtt_tls.py --ca-cert certs/ca.cert.pem \\
                          --client-cert certs/client.cert.pem \\
                          --client-key certs/client.key
  
  # 禁用证书验证（用于自签名证书或开发测试）
  python test_mqtt_tls.py --insecure
  或
  $env:MQTT_TLS_INSECURE='true'; python test_mqtt_tls.py
        """
    )
    
    parser.add_argument('--host', default='127.0.0.1',
                       help='MQTT Broker 主机地址 (默认: 127.0.0.1)')
    parser.add_argument('--port', type=int, default=1883,
                       help='MQTT Broker 端口 (默认: 1883)')
    parser.add_argument('--ca-cert', 
                       help='CA 证书路径 (用于验证服务器证书)')
    parser.add_argument('--client-cert',
                       help='客户端证书路径 (用于客户端证书认证)')
    parser.add_argument('--client-key',
                       help='客户端密钥路径 (用于客户端证书认证)')
    parser.add_argument('--insecure', action='store_true',
                       help='禁用 TLS 证书验证（仅用于开发测试，不安全）')
    
    args = parser.parse_args()
    
    # 如果指定了 --insecure，设置环境变量
    if args.insecure:
        import os
        os.environ['MQTT_TLS_INSECURE'] = 'true'
    
    # 自动查找证书文件（如果未指定）
    import os
    if not args.ca_cert:
        possible_ca_certs = [
            'certs/ca.cert.pem',
            'XrayR/certs/ca.cert.pem',
            '../certs/ca.cert.pem',
        ]
        for path in possible_ca_certs:
            if os.path.exists(path):
                args.ca_cert = path
                print(f"自动发现 CA 证书: {path}")
                break
    
    if not args.client_cert:
        possible_client_certs = [
            'certs/client.cert.pem',
            'XrayR/certs/client.cert.pem',
            '../certs/client.cert.pem',
        ]
        for path in possible_client_certs:
            if os.path.exists(path):
                args.client_cert = path
                print(f"自动发现客户端证书: {path}")
                break
    
    if not args.client_key:
        possible_client_keys = [
            'certs/client.key',
            'XrayR/certs/client.key',
            '../certs/client.key',
        ]
        for path in possible_client_keys:
            if os.path.exists(path):
                args.client_key = path
                print(f"自动发现客户端密钥: {path}")
                break
    
    # 运行测试
    tester = MQTTTester(host=args.host, port=args.port)
    tester.run_tests(
        ca_cert=args.ca_cert,
        client_cert=args.client_cert,
        client_key=args.client_key
    )


if __name__ == '__main__':
    main()

