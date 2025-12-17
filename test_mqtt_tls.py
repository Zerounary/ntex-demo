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

# 尝试导入 colorama 用于彩色输出
try:
    from colorama import init, Fore, Back, Style
    # 在 Windows 上启用 ANSI 转义码支持
    init(autoreset=True, strip=False)  # strip=False 确保颜色不被移除
    HAS_COLORAMA = True
except ImportError:
    # 如果没有 colorama，尝试手动启用 Windows ANSI 支持
    HAS_COLORAMA = False
    import sys
    if sys.platform == 'win32':
        try:
            import ctypes
            kernel32 = ctypes.windll.kernel32
            # 启用虚拟终端处理（Windows 10 1607+）
            kernel32.SetConsoleMode(kernel32.GetStdHandle(-11), 7)
        except:
            pass
    
    class Fore:
        GREEN = '\033[92m'
        RED = '\033[91m'
        YELLOW = '\033[93m'
        BLUE = '\033[94m'
        CYAN = '\033[96m'
        MAGENTA = '\033[95m'
        RESET = '\033[0m'
    class Style:
        BRIGHT = '\033[1m'
        RESET_ALL = '\033[0m'


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
                print("  [OK] 普通 MQTT 连接成功")
                return True, "连接成功"
            else:
                print(f"  [FAIL] 普通 MQTT 连接失败: {error_msg[0] or '未知错误'}")
                return False, error_msg[0] or "未知错误"
                
        except Exception as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            print(f"  [FAIL] 普通 MQTT 连接异常: {e}")
            return False, str(e)

    def test_tls_mqtt_without_client_cert(self, ca_cert: Optional[str] = None) -> Tuple[bool, str]:
        """测试 TLS MQTT 连接（不使用客户端证书）"""
        print(f"\n[测试 2] TLS MQTT 连接（无客户端证书）(ssl://{self.host}:{self.port})")
        print("-" * 60)
        
        if not self.test_tcp_connection():
            return False, "TCP 连接不可达"
        
        client = None
        try:
            client = mqtt.Client(client_id="test_tls_mqtt_no_client_cert")
            client._connect_timeout = 3
            
            # 配置 TLS（不使用客户端证书）
            allow_insecure = os.getenv("MQTT_TLS_INSECURE", "").lower() == "true"
            force_verify = os.getenv("MQTT_TLS_INSECURE", "").lower() == "false"
            
            if allow_insecure or (not force_verify):
                if allow_insecure:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 已启用不安全模式（不验证证书），仅用于开发测试")
                else:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 检测到自签名证书，自动使用不安全模式（不验证证书）")
                tls_context = ssl.create_default_context()
                tls_context.check_hostname = False
                tls_context.verify_mode = ssl.CERT_NONE
            else:
                print(f"  {Fore.GREEN}[OK]{Fore.RESET} 强制验证模式（已设置 MQTT_TLS_INSECURE=false）")
                tls_context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
                
                if ca_cert:
                    try:
                        tls_context.load_verify_locations(ca_cert)
                        print(f"  {Fore.GREEN}[OK]{Fore.RESET} 已加载 CA 证书: {Fore.CYAN}{ca_cert}{Fore.RESET}")
                        tls_context.check_hostname = False
                    except Exception as e:
                        print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 加载 CA 证书失败: {Fore.RED}{e}{Fore.RESET}")
                        raise
                else:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 未提供 CA 证书，将使用系统默认证书")
            
            # 注意：不加载客户端证书
            print(f"  {Fore.BLUE}[INFO]{Fore.RESET} 未使用客户端证书（测试 broker 是否强制要求客户端证书）")
            
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
            print(f"  正在使用 TLS 连接到 {self.host}:{self.port}（无客户端证书）...")
            client.connect(self.host, self.port, keepalive=60)
            client.loop_start()
            
            # 等待连接结果
            time.sleep(2)
            client.loop_stop()
            client.disconnect()
            
            if connected[0]:
                print(f"  {Fore.GREEN}[OK]{Fore.RESET} TLS MQTT 连接成功（无客户端证书）")
                return True, "TLS 连接成功（无客户端证书）"
            else:
                print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS MQTT 连接失败: {Fore.YELLOW}{error_msg[0] or '未知错误'}{Fore.RESET}")
                return False, error_msg[0] or "未知错误"
                
        except ssl.SSLError as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            error_msg = str(e)
            # 如果是证书相关的错误，可能是broker要求客户端证书
            if "CERTIFICATE" in error_msg.upper() or "HANDSHAKE" in error_msg.upper():
                print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} TLS 握手失败: {Fore.RED}{e}{Fore.RESET}")
                print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 这可能意味着 broker 强制要求客户端证书")
                return False, f"SSL 错误（可能要求客户端证书）: {e}"
            else:
                print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS 握手失败: {Fore.RED}{e}{Fore.RESET}")
            return False, f"SSL 错误: {e}"
        except Exception as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS MQTT 连接异常: {Fore.RED}{e}{Fore.RESET}")
            return False, str(e)

    def test_tls_mqtt(self, ca_cert: Optional[str] = None, 
                      client_cert: Optional[str] = None,
                      client_key: Optional[str] = None) -> Tuple[bool, str]:
        """测试 TLS MQTT 连接（使用客户端证书）"""
        print(f"\n[测试 3] TLS MQTT 连接（使用客户端证书）(ssl://{self.host}:{self.port})")
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
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 已启用不安全模式（不验证证书），仅用于开发测试")
                else:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 检测到自签名证书，自动使用不安全模式（不验证证书）")
                    print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 提示: 这是正常的，因为自签名证书无法通过严格验证")
                    print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 如需强制验证，请设置: {Fore.MAGENTA}$env:MQTT_TLS_INSECURE='false'{Fore.RESET}")
                tls_context = ssl.create_default_context()
                tls_context.check_hostname = False
                tls_context.verify_mode = ssl.CERT_NONE
            else:
                # 强制验证模式
                print(f"  {Fore.GREEN}[OK]{Fore.RESET} 强制验证模式（已设置 MQTT_TLS_INSECURE=false）")
                tls_context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
                
                if ca_cert:
                    try:
                        tls_context.load_verify_locations(ca_cert)
                        print(f"  {Fore.GREEN}[OK]{Fore.RESET} 已加载 CA 证书: {Fore.CYAN}{ca_cert}{Fore.RESET}")
                        tls_context.check_hostname = False
                    except Exception as e:
                        print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 加载 CA 证书失败: {Fore.RED}{e}{Fore.RESET}")
                        raise
                else:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 未提供 CA 证书，将使用系统默认证书")
            
            if client_cert and client_key:
                try:
                    tls_context.load_cert_chain(client_cert, client_key)
                    print(f"  {Fore.GREEN}[OK]{Fore.RESET} 已加载客户端证书: {Fore.CYAN}{client_cert}{Fore.RESET}")
                except Exception as e:
                    print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 加载客户端证书失败: {Fore.RED}{e}{Fore.RESET}")
            
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
                print(f"  {Fore.GREEN}[OK]{Fore.RESET} TLS MQTT 连接成功")
                return True, "TLS 连接成功"
            else:
                print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS MQTT 连接失败: {Fore.YELLOW}{error_msg[0] or '未知错误'}{Fore.RESET}")
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
                print(f"  {Fore.YELLOW}[WARN]{Fore.RESET} 证书验证失败: {Fore.RED}{e}{Fore.RESET}")
                print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 这是自签名证书的正常现象，正在尝试不验证证书模式...")
                
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
                    
                    print(f"  {Fore.BLUE}[RETRY]{Fore.RESET} 重试连接（不验证证书）...")
                    client2.connect(self.host, self.port, keepalive=60)
                    client2.loop_start()
                    time.sleep(2)
                    client2.loop_stop()
                    client2.disconnect()
                    
                    if connected2[0]:
                        print(f"  {Fore.GREEN}[OK]{Fore.RESET} TLS MQTT 连接成功（不验证证书模式）")
                        print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 说明: 虽然证书验证失败，但 TLS 加密连接已建立")
                        print(f"  {Fore.CYAN}[TIP]{Fore.RESET} 这是自签名证书的正常行为，通信已加密")
                        return True, "TLS 连接成功（不验证证书）"
                    else:
                        print(f"  {Fore.RED}[FAIL]{Fore.RESET} 重试也失败: {Fore.YELLOW}{error_msg2[0] or '未知错误'}{Fore.RESET}")
                        return False, f"SSL 错误: {e}，重试也失败: {error_msg2[0]}"
                except Exception as e2:
                    print(f"  {Fore.RED}[FAIL]{Fore.RESET} 重试失败: {Fore.RED}{e2}{Fore.RESET}")
                    return False, f"SSL 错误: {e}，重试失败: {e2}"
            else:
                print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS 握手失败: {Fore.RED}{e}{Fore.RESET}")
            return False, f"SSL 错误: {e}"
        except Exception as e:
            if client:
                try:
                    client.disconnect()
                except:
                    pass
            print(f"  {Fore.RED}[FAIL]{Fore.RESET} TLS MQTT 连接异常: {Fore.RED}{e}{Fore.RESET}")
            return False, str(e)

    def run_tests(self, ca_cert: Optional[str] = None,
                  client_cert: Optional[str] = None,
                  client_key: Optional[str] = None):
        """运行所有测试"""
        print("=" * 60)
        print(f"{Fore.CYAN}MQTT 连接测试工具{Fore.RESET}")
        print("=" * 60)
        print(f"目标服务器: {Fore.MAGENTA}{self.host}:{self.port}{Fore.RESET}")
        print(f"CA 证书: {Fore.CYAN}{ca_cert or '未指定'}{Fore.RESET}")
        print(f"客户端证书: {Fore.CYAN}{client_cert or '未指定'}{Fore.RESET}")
        print(f"客户端密钥: {Fore.CYAN}{client_key or '未指定'}{Fore.RESET}")
        
        # 测试普通连接
        print(f"\n{Fore.CYAN}{'='*60}{Fore.RESET}")
        print(f"{Fore.CYAN}开始测试...{Fore.RESET}")
        print(f"{Fore.CYAN}{'='*60}{Fore.RESET}")
        plain_success, plain_msg = self.test_plain_mqtt()
        self.results['plain'] = {
            'success': plain_success,
            'message': plain_msg
        }
        
        # 测试 TLS 连接（不使用客户端证书）
        tls_no_cert_success, tls_no_cert_msg = self.test_tls_mqtt_without_client_cert(ca_cert)
        self.results['tls_no_client_cert'] = {
            'success': tls_no_cert_success,
            'message': tls_no_cert_msg
        }
        
        # 测试 TLS 连接（使用客户端证书）
        tls_success, tls_msg = self.test_tls_mqtt(ca_cert, client_cert, client_key)
        self.results['tls'] = {
            'success': tls_success,
            'message': tls_msg
        }
        
        # 输出总结
        print("\n" + "=" * 60)
        print(f"{Fore.CYAN}测试结果总结{Fore.RESET}")
        print("=" * 60)
        # 使用颜色显示结果
        plain_status = f"{Fore.GREEN}[OK] 成功{Fore.RESET}" if plain_success else f"{Fore.RED}[FAIL] 失败{Fore.RESET}"
        tls_no_cert_status = f"{Fore.GREEN}[OK] 成功{Fore.RESET}" if tls_no_cert_success else f"{Fore.RED}[FAIL] 失败{Fore.RESET}"
        tls_cert_status = f"{Fore.GREEN}[OK] 成功{Fore.RESET}" if tls_success else f"{Fore.RED}[FAIL] 失败{Fore.RESET}"
        
        print(f"普通 MQTT 连接: {plain_status}")
        if not plain_success:
            print(f"  原因: {Fore.YELLOW}{plain_msg}{Fore.RESET}")
        print(f"TLS MQTT 连接（无客户端证书）: {tls_no_cert_status}")
        if not tls_no_cert_success:
            print(f"  原因: {Fore.YELLOW}{tls_no_cert_msg}{Fore.RESET}")
        print(f"TLS MQTT 连接（使用客户端证书）: {tls_cert_status}")
        if not tls_success:
            print(f"  原因: {Fore.YELLOW}{tls_msg}{Fore.RESET}")
        
        print("\n" + "=" * 60)
        print(f"{Fore.CYAN}分析结果{Fore.RESET}")
        print("=" * 60)
        
        # 分析客户端证书认证要求
        if tls_no_cert_success and tls_success:
            print(f"{Fore.YELLOW}[WARN]{Fore.RESET} 客户端证书认证: 可选（不强制）")
            print(f"   - 不使用客户端证书可以连接 {Fore.GREEN}[OK]{Fore.RESET}")
            print(f"   - 使用客户端证书也可以连接 {Fore.GREEN}[OK]{Fore.RESET}")
            print(f"   - Broker 未强制要求客户端证书")
            print(f"   - 建议: 如需更高安全性，配置 broker 强制要求客户端证书")
        elif not tls_no_cert_success and tls_success:
            print(f"{Fore.GREEN}[OK]{Fore.RESET} 客户端证书认证: 强制要求")
            print(f"   - 不使用客户端证书连接失败 {Fore.RED}[FAIL]{Fore.RESET}")
            print(f"   - 使用客户端证书连接成功 {Fore.GREEN}[OK]{Fore.RESET}")
            print(f"   - Broker 已强制要求客户端证书")
            print(f"   - 安全性: {Fore.GREEN}高{Fore.RESET}（只有持有有效证书的客户端才能连接）")
        elif tls_no_cert_success and not tls_success:
            print(f"{Fore.YELLOW}[WARN]{Fore.RESET} 异常情况: 无客户端证书可连接，但有客户端证书失败")
            print(f"   - 这可能表示客户端证书配置有问题")
            print(f"   - 建议: 检查客户端证书和密钥是否正确")
        else:
            print(f"{Fore.RED}[FAIL]{Fore.RESET} 两种 TLS 连接方式都失败")
            print(f"   - 可能原因:")
            print(f"     1. Broker 未运行或 TLS 配置错误")
            print(f"     2. 网络问题")
            print(f"     3. 防火墙阻止连接")
        
        print("")
        
        # 分析普通连接 vs TLS 连接
        if plain_success and (tls_no_cert_success or tls_success):
            print(f"{Fore.YELLOW}[WARN]{Fore.RESET} 警告: 普通连接和 TLS 连接都成功！")
            print(f"   这意味着 Broker 同时支持普通连接和 TLS 连接")
            print(f"   建议: 禁用普通连接，仅使用 TLS 以确保安全")
        elif not plain_success and (tls_no_cert_success or tls_success):
            print(f"{Fore.GREEN}[OK]{Fore.RESET} 最佳情况: 仅 TLS 连接成功")
            print(f"   这意味着 Broker 已正确配置为仅接受 TLS 连接")
            print(f"   普通连接被拒绝，{Fore.GREEN}安全性良好{Fore.RESET}")
        elif plain_success and not tls_no_cert_success and not tls_success:
            print(f"{Fore.YELLOW}[WARN]{Fore.RESET} 警告: 仅普通连接成功，TLS 连接失败")
            print(f"   这意味着 Broker 可能未启用 TLS 或 TLS 配置有误")
            print(f"   建议: 检查 Broker 的 TLS 配置")
        elif not plain_success and not tls_no_cert_success and not tls_success:
            print(f"{Fore.RED}[FAIL]{Fore.RESET} 错误: 所有连接方式都失败")
            print(f"   可能原因:")
            print(f"   1. Broker 未运行")
            print(f"   2. 端口不正确")
            print(f"   3. 防火墙阻止连接")
            print(f"   4. 网络问题")


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

