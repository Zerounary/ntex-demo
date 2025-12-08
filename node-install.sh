#!/bin/bash

###############################################################################
# 游戏节点自动部署脚本
# 功能：
# 1. 安装和配置 XrayR
# 2. 自动配置防火墙
# 3. 向后台服务器注册节点并推送 API token
###############################################################################

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置变量（需要根据实际情况修改）
SERVER_URL="${SERVER_URL:-http://localhost:8080}"  # 后台服务器地址
API_TOKEN="${API_TOKEN:-}"  # API Token（可选，如果服务器需要认证）
XRAYR_API_PORT="${XRAYR_API_PORT:-54321}"  # XrayR API 端口
NODE_ID="${NODE_ID:-}"  # 节点ID（可选，如果不提供则自动生成）

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否为 root 用户
check_root() {
    if [ "$EUID" -ne 0 ]; then 
        log_error "请使用 root 权限运行此脚本"
        exit 1
    fi
}

# 检测系统类型
detect_system() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        VER=$VERSION_ID
    else
        log_error "无法检测系统类型"
        exit 1
    fi
    
    log_info "检测到系统: $OS $VER"
}

# 检测防火墙类型
detect_firewall() {
    if command -v ufw &> /dev/null; then
        FIREWALL="ufw"
    elif command -v firewall-cmd &> /dev/null; then
        FIREWALL="firewalld"
    elif command -v iptables &> /dev/null; then
        FIREWALL="iptables"
    else
        log_warn "未检测到防火墙，跳过防火墙配置"
        FIREWALL="none"
    fi
    
    log_info "检测到防火墙: $FIREWALL"
}

# 安装 XrayR
install_xrayr() {
    log_info "开始安装 XrayR..."
    
    if command -v XrayR &> /dev/null; then
        log_warn "XrayR 已安装，跳过安装步骤"
        return
    fi
    
    # 使用官方一键安装脚本
    bash <(curl -Ls https://raw.githubusercontent.com/XrayR-project/XrayR-release/master/install.sh)
    
    if [ $? -eq 0 ]; then
        log_info "XrayR 安装成功"
    else
        log_error "XrayR 安装失败"
        exit 1
    fi
}

# 配置防火墙
configure_firewall() {
    if [ "$FIREWALL" = "none" ]; then
        return
    fi
    
    log_info "配置防火墙，开放端口 $XRAYR_API_PORT..."
    
    case $FIREWALL in
        ufw)
            ufw allow $XRAYR_API_PORT/tcp comment "XrayR API"
            ufw --force enable
            ;;
        firewalld)
            firewall-cmd --permanent --add-port=$XRAYR_API_PORT/tcp
            firewall-cmd --reload
            ;;
        iptables)
            iptables -I INPUT -p tcp --dport $XRAYR_API_PORT -j ACCEPT
            # 保存规则（根据系统不同可能需要不同命令）
            if command -v iptables-save &> /dev/null; then
                iptables-save > /etc/iptables/rules.v4 2>/dev/null || \
                iptables-save > /etc/iptables.rules 2>/dev/null || true
            fi
            ;;
    esac
    
    log_info "防火墙配置完成"
}

# 读取 XrayR 配置
read_xrayr_config() {
    local config_file="/etc/XrayR/config.yml"
    
    if [ ! -f "$config_file" ]; then
        log_error "XrayR 配置文件不存在: $config_file"
        log_info "请先配置 XrayR，配置文件路径: $config_file"
        exit 1
    fi
    
    log_info "读取 XrayR 配置..."
    
    # 使用 yq 或 python 解析 YAML（如果没有 yq，使用 python）
    if command -v yq &> /dev/null; then
        # 读取 API 端口
        local api_port=$(yq eval '.ApiConfig.Listen' "$config_file" 2>/dev/null || echo "")
        if [ -n "$api_port" ] && [ "$api_port" != "null" ] && [ "$api_port" != "" ]; then
            # 处理格式如 "0.0.0.0:54321" 或 ":54321"
            if echo "$api_port" | grep -q ':'; then
                XRAYR_API_PORT=$(echo "$api_port" | awk -F':' '{print $NF}' | grep -oE '[0-9]+' | head -1)
            else
                XRAYR_API_PORT=$(echo "$api_port" | grep -oE '[0-9]+' | head -1)
            fi
            [ -z "$XRAYR_API_PORT" ] && XRAYR_API_PORT="${XRAYR_API_PORT:-54321}"
        fi
        
        # 读取 API Token
        local api_token=$(yq eval '.ApiConfig.Token' "$config_file" 2>/dev/null || echo "")
        if [ -n "$api_token" ] && [ "$api_token" != "null" ] && [ "$api_token" != "" ]; then
            API_TOKEN="$api_token"
        fi
        
        # 读取节点信息（从第一个节点配置）
        VMESS_UUID=$(yq eval '.Nodes[0].VmessSettings.UUID' "$config_file" 2>/dev/null || echo "")
        VMESS_PORT=$(yq eval '.Nodes[0].VmessSettings.Port' "$config_file" 2>/dev/null || echo "")
        VMESS_EMAIL=$(yq eval '.Nodes[0].VmessSettings.Email' "$config_file" 2>/dev/null || echo "")
        
    elif command -v python3 &> /dev/null; then
        # 使用 Python 解析 YAML
        local config_data=$(python3 << 'EOF'
import yaml
import sys
import json

try:
    with open('/etc/XrayR/config.yml', 'r') as f:
        config = yaml.safe_load(f)
    
    result = {}
    
    # API 配置
    if 'ApiConfig' in config:
        api_config = config['ApiConfig']
        if 'Listen' in api_config:
            listen = api_config['Listen']
            if isinstance(listen, str) and ':' in listen:
                result['api_port'] = listen.split(':')[-1]
            elif isinstance(listen, int):
                result['api_port'] = str(listen)
        if 'Token' in api_config:
            result['api_token'] = api_config['Token']
    
    # 节点配置
    if 'Nodes' in config and len(config['Nodes']) > 0:
        node = config['Nodes'][0]
        if 'VmessSettings' in node:
            vmess = node['VmessSettings']
            if 'UUID' in vmess:
                result['vmess_uuid'] = vmess['UUID']
            if 'Port' in vmess:
                result['vmess_port'] = str(vmess['Port'])
            if 'Email' in vmess:
                result['vmess_email'] = vmess['Email']
    
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({}))
    sys.exit(1)
EOF
)
        
        if [ -n "$config_data" ]; then
            XRAYR_API_PORT=$(echo "$config_data" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('api_port', '$XRAYR_API_PORT'))")
            API_TOKEN=$(echo "$config_data" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('api_token', ''))")
            VMESS_UUID=$(echo "$config_data" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('vmess_uuid', ''))")
            VMESS_PORT=$(echo "$config_data" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('vmess_port', ''))")
            VMESS_EMAIL=$(echo "$config_data" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('vmess_email', ''))")
        fi
    else
        log_warn "未找到 yq 或 python3，无法自动解析配置"
        log_warn "请手动设置以下环境变量："
        log_warn "  VMESS_UUID, VMESS_PORT, VMESS_EMAIL, XRAYR_API_PORT"
        log_warn "或者安装 yq: wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64"
    fi
    
    # 验证必要参数
    if [ -z "$VMESS_UUID" ] || [ -z "$VMESS_PORT" ]; then
        log_error "无法从配置文件读取必要的节点信息"
        log_error "请确保 XrayR 配置文件包含完整的节点配置"
        log_error "配置文件路径: $config_file"
        exit 1
    fi
    
    # 获取服务器 IP
    VMESS_SERVER=$(curl -s ifconfig.me || curl -s ip.sb || hostname -I | awk '{print $1}' || echo "127.0.0.1")
    
    # 生成节点 ID（如果未提供）
    if [ -z "$NODE_ID" ]; then
        NODE_ID="node-$(hostname)-$(date +%s)"
    fi
    
    log_info "节点配置信息:"
    log_info "  节点ID: $NODE_ID"
    log_info "  服务器IP: $VMESS_SERVER"
    log_info "  VMess端口: $VMESS_PORT"
    log_info "  API端口: $XRAYR_API_PORT"
}

# 启动 XrayR
start_xrayr() {
    log_info "启动 XrayR 服务..."
    
    systemctl enable XrayR
    systemctl restart XrayR
    
    # 等待服务启动
    sleep 3
    
    if systemctl is-active --quiet XrayR; then
        log_info "XrayR 服务启动成功"
    else
        log_error "XrayR 服务启动失败"
        systemctl status XrayR
        exit 1
    fi
}

# 向服务器注册节点
register_node() {
    log_info "向服务器注册节点: $SERVER_URL"
    
    # 检查必要参数
    if [ -z "$VMESS_UUID" ] || [ -z "$VMESS_PORT" ]; then
        log_error "缺少必要的节点配置信息"
        log_error "请确保 XrayR 配置文件包含完整的节点信息"
        exit 1
    fi
    
    # 计算延迟（ping 测试）
    PING_MS=0
    if command -v ping &> /dev/null; then
        # 尝试 ping 服务器（如果可访问）
        local server_host=$(echo "$SERVER_URL" | sed -E 's|https?://([^/]+).*|\1|' | cut -d: -f1)
        if [ -n "$server_host" ] && [ "$server_host" != "localhost" ] && [ "$server_host" != "127.0.0.1" ]; then
            PING_MS=$(ping -c 3 -W 2 "$server_host" 2>/dev/null | grep 'avg' | awk -F'/' '{print $5}' | cut -d. -f1 || echo "0")
        fi
    fi
    
    # 构建注册请求
    local register_data=$(cat <<EOF
{
  "id": "$NODE_ID",
  "vmessUuid": "$VMESS_UUID",
  "vmessServer": "$VMESS_SERVER",
  "vmessPort": $VMESS_PORT,
  "vmessEmail": "${VMESS_EMAIL:-default@node.local}",
  "udpProxy": "$VMESS_SERVER:$VMESS_PORT",
  "mode": "进程模式",
  "ping": $PING_MS,
  "status": "active"
}
EOF
)
    
    # 发送注册请求
    local response
    if [ -n "$API_TOKEN" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $API_TOKEN" \
            -d "$register_data" \
            "$SERVER_URL/api/nodes/register" 2>/dev/null)
    else
        response=$(curl -s -w "\n%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -d "$register_data" \
            "$SERVER_URL/api/nodes/register" 2>/dev/null)
    fi
    
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
        log_info "节点注册成功"
        echo "$body" | python3 -m json.tool 2>/dev/null || echo "$body"
    else
        log_error "节点注册失败 (HTTP $http_code)"
        echo "$body"
        exit 1
    fi
}

# 设置定时心跳任务
setup_heartbeat() {
    log_info "设置定时心跳任务..."
    
    local heartbeat_script="/usr/local/bin/xrayr-heartbeat.sh"
    
    # 创建心跳脚本
    cat > "$heartbeat_script" <<EOF
#!/bin/bash
# XrayR 节点心跳脚本

NODE_ID="$NODE_ID"
SERVER_URL="$SERVER_URL"
API_TOKEN="$API_TOKEN"

curl -s -X POST \\
    ${API_TOKEN:+-H "Authorization: Bearer \$API_TOKEN"} \\
    "$SERVER_URL/api/nodes/\$NODE_ID/heartbeat" > /dev/null 2>&1

if [ \$? -eq 0 ]; then
    echo "\$(date): Heartbeat sent successfully" >> /var/log/xrayr-heartbeat.log
else
    echo "\$(date): Heartbeat failed" >> /var/log/xrayr-heartbeat.log
fi
EOF
    
    chmod +x "$heartbeat_script"
    
    # 添加到 crontab（每5分钟执行一次）
    (crontab -l 2>/dev/null | grep -v "xrayr-heartbeat"; echo "*/5 * * * * $heartbeat_script") | crontab -
    
    log_info "心跳任务已设置（每5分钟执行一次）"
}

# 主函数
main() {
    log_info "=========================================="
    log_info "游戏节点自动部署脚本"
    log_info "=========================================="
    
    check_root
    detect_system
    detect_firewall
    
    # 安装 XrayR
    install_xrayr
    
    # 读取配置
    read_xrayr_config
    
    # 配置防火墙
    configure_firewall
    
    # 启动 XrayR
    start_xrayr
    
    # 注册节点
    register_node
    
    # 设置心跳
    setup_heartbeat
    
    log_info "=========================================="
    log_info "部署完成！"
    log_info "节点ID: $NODE_ID"
    log_info "服务器: $SERVER_URL"
    log_info "=========================================="
}

# 运行主函数
main "$@"

