use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, JsonValue, PaginatorTrait, Set};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::infrastructure::persistence::{
    accelerator_game, accelerator_node, accelerator_profile, accelerator_user, account_user,
    accelerator_user_credential,
    admin_inbound, admin_node_config, admin_outbound, admin_routing, admin_user, admin_user_mapping,
    config_entry,
};

pub async fn seed(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    seed_accelerator(db).await?;
    seed_config_entries(db).await?;
    seed_accounts(db).await?;
    seed_admin_config(db).await?;
    Ok(())
}

async fn seed_accelerator(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let game_count = accelerator_game::Entity::find().count(db).await?;
    if game_count == 0 {
        let games = vec![accelerator_game::ActiveModel {
            id: Set("7".to_string()),
            name: Set("Chrome 加速测试".to_string()),
            icon: Set("i-mdi-google-chrome".to_string()),
            status: Set("idle".to_string()),
            ping: Set(0),
            process_name: Set("chrome.exe,udptest.exe,LinkLatencyTester.exe".to_string()),
            routing_rules: Set(Some("[]".to_string())),
            sniff_domains_excluded: Set(Some("[]".to_string())),
            region: Set("测试".to_string()),
        },
        accelerator_game::ActiveModel {
            id: Set("8".to_string()),
            name: Set("LOL 韩服".to_string()),
            icon: Set("i-mdi-google-chrome".to_string()),
            status: Set("idle".to_string()),
            ping: Set(0),
            process_name: Set("League of Legends.exe,LeagueClient.exe,LeagueClientUx.exe,LolClient.exe,LoLLauncher.exe,LoLPatcher.exe,LoLPatcherUx.exe,RiotClientServices.exe,RiotClientUx.exe,RiotClientUxRender.exe,LeagueClientUxRender.exe,Garena.exe,GarenaMessenger.exe,lol.exe,rads_user_kernel.exe,client.exe,LeagueCrashHandler.exe,RuinedKing.exe,steamwebhelper.exe,EALink.exe,Client.exe,Riot Client.exe,OP.GG.exe,udptest.exe,vgc.exe,chrome.exe".to_string()),
            routing_rules: Set(Some(
                json!([
                    {
                        "type": "field",
                        "outboundTag": "direct",
                        "domain": [
                            "domain:lol.dyn.riotcdn.net",
                            "domain:legendspatch-lol.cdn.x-cdn.com",
                            "domain:lol.secure.dyn.riotcdn.net",
                            "domain:123cha.com",
                            "domain:ip112.com"
                        ],
                        "enabled": true
                    }
                ])
                .to_string(),
            )),
            sniff_domains_excluded: Set(Some(
                json!([
                    "rbsxbxp-mim.vivox.com",
                    "rbsxbxp.www.vivox.com",
                    "rbsxbxp-ws.vivox.com",
                    "rbspsxp.www.vivox.com",
                    "rbspsxp-mim.vivox.com",
                    "rbspsxp-ws.vivox.com",
                    "rbswxp.www.vivox.com",
                    "rbswxp-mim.vivox.com",
                    "disp-rbspsp-5-1.vivox.com",
                    "disp-rbsxbp-5-1.vivox.com",
                    "proxy.rbsxbp.vivox.com",
                    "proxy.rbspsp.vivox.com",
                    "proxy.rbswp.vivox.com",
                    "rbswp.vivox.com",
                    "rbsxbp.vivox.com",
                    "rbspsp.vivox.com",
                    "rbspsp.www.vivox.com",
                    "rbswp.www.vivox.com",
                    "rbsxbp.www.vivox.com",
                    "rbsxbxp.vivox.com",
                    "rbspsxp.vivox.com",
                    "rbswxp.vivox.com",
                    "pubwxp.vivox.com",
                    "lolsnxp.vivox.com",
                    "lolsnxp.www.vivox.com",
                    "lolsnxp-mim1.vivox.com",
                    "lolsnxp-mim2.vivox.com"
                ])
                .to_string(),
            )),
            region: Set("韩国".to_string()),
        }];

        for game in games {
            game.insert(db).await?;
        }
    }

    let node_count = accelerator_node::Entity::find().count(db).await?;
    if node_count == 0 {
        let nodes = vec![accelerator_node::ActiveModel {
            id: Set("1".to_string()),
            vless_id: Set("9acea125-3ca7-1212-2121-000000010135".to_string()),
            vless_server: Set("127.0.0.1".to_string()),
            vless_port: Set(10086),
            vless_encryption: Set("none".to_string()),
            reality_server_name: Set("www.cloudflare.com".to_string()),
            reality_public_key: Set("JbPBpbjEHQiL87HpoJ6wZ3o9wSTyjzTIN9ysP6tnywU".to_string()),
            reality_short_id: Set("a66cafe7".to_string()),
            reality_fingerprint: Set("chrome".to_string()),
            reality_spider_x: Set("/".to_string()),
            udp_proxy: Set("127.0.0.1:10086".to_string()),
            mode: Set("进程模式".to_string()),
            ping: Set(5),
            status: Set("active".to_string()),
            last_heartbeat: Set(Utc::now().into()),
        }];

        for node in nodes {
            node.insert(db).await?;
        }
    }

    let profile_count = accelerator_profile::Entity::find().count(db).await?;
    if profile_count == 0 {
        let profiles = vec![accelerator_profile::ActiveModel {
            id: Set("profile-chrome-test".to_string()),
            game_id: Set("7".to_string()),
            display_name: Set("Chrome · 调试隧道".to_string()),
            node_id: Set("1".to_string()),
            status: Set("空闲".to_string()),
        }];

        for profile in profiles {
            profile.insert(db).await?;
        }
    }

    let user_count = accelerator_user::Entity::find().count(db).await?;
    if user_count == 0 {
        let now = Utc::now();
        let email = "test@example.com".to_string();
        let inserted = accelerator_user::ActiveModel {
            id: sea_orm::NotSet,
            email: Set(email.clone()),
            name: Set("User".to_string()),
            invite_code: Set("TESTINVITE".to_string()),
            inviter_id: Set(None),
            valid_until: Set((now + Duration::days(365)).into()),
        }
        .insert(db)
        .await?;
        let user_id = inserted.id;

        // 默认密码与 account_users 一致：secret
        accelerator_user_credential::ActiveModel {
            user_id: Set(user_id),
            password_hash: Set(hash_password("secret")),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(db)
        .await?;
    }

    Ok(())
}

async fn seed_config_entries(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let entries = vec![
        ("dashboard", dashboard_payload()),
        ("library", library_payload()),
        ("settings_meta", settings_payload()),
        ("navigation", navigation_payload()),
        ("accelerator_activity", accelerator_activity_payload()),
        ("user_register_activity", user_register_activity_payload()),
        ("entry_config", entry_config_payload()),
    ];

    for (key, payload) in entries {
        let exists = config_entry::Entity::find_by_id(key.to_string())
            .one(db)
            .await?;
        if exists.is_none() {
            config_entry::ActiveModel {
                key: Set(key.to_string()),
                payload: Set(payload),
                updated_at: Set(Utc::now().into()),
            }
            .insert(db)
            .await?;
        }
    }

    Ok(())
}

fn dashboard_payload() -> JsonValue {
    json!({
        "announcements": [
            { "id": 1, "title": "【维护】2月全球节点例行维护", "date": "12:30" }
        ],
        "heroStats": [
            { "label": "在线节点", "value": "128", "hint": "+12 新增" }
        ],
        "quickPanels": [
            { "title": "量子加速", "desc": "独占物理专线", "icon": "i-mdi-flash", "accent": "rgba(255,255,255,0.25)" }
        ],
        "featuredSnapshots": [
            { "label": "亚服 · 旗舰节点", "value": "B1-东京 4001", "signal": "低负载" }
        ],
        "marqueeItems": [
            { "label": "NEBULA CORE", "value": "SYNC 99.98%" }
        ],
        "hologramGlyphs": ["Δ", "Ω", "Ξ", "⌁", "Φ", "∑"]
    })
}

fn library_payload() -> JsonValue {
    json!({
        "categories": ["最新上线", "全部", "热门", "限免", "Steam", "橘子", "暴雪", "Epic"],
        "curatedCollections": [
            { "title": "对战优选", "desc": "FPS/竞技类低延迟专线", "value": "12 条" }
        ],
        "opsMemos": [
            { "label": "节点巡航", "detail": "亚洲集群调度完成" }
        ]
    })
}

fn user_register_activity_payload() -> JsonValue {
    json!({
        "enabled": true,
        "cdkType": "day",
        "num": 3
    })
}

fn settings_payload() -> JsonValue {
    json!({
        "version": "v1.0.0",
        "statusLabel": "LIVE",
        "statusDescription": "节点自检已完成",
        "heroStats": [
            { "label": "授权版本", "value": "Stable", "hint": "最新渠道" },
            { "label": "巡航状态", "value": "在线", "hint": "节点巡航中" },
            { "label": "通知中心", "value": "启用", "hint": "安全提醒开启" }
        ],
        "regions": [
            { "value": "auto", "label": "自动选择 (推荐)" },
            { "value": "asia", "label": "亚太地区" },
            { "value": "na", "label": "北美地区" },
            { "value": "eu", "label": "欧洲地区" }
        ]
    })
}

fn navigation_payload() -> JsonValue {
    json!({
        "menu": [
            { "name": "首页", "path": "/", "icon": "i-mdi-home-variant-outline" },
            { "name": "我的加速", "path": "/my-boosts", "icon": "i-mdi-rocket-launch-outline" },
            { "name": "游戏库", "path": "/library", "icon": "i-mdi-gamepad-variant-outline" }
        ]
    })
}

fn accelerator_activity_payload() -> JsonValue {
    json!({
        "enabled": true,
        "inviteRewards": [
            {
                "inviterCount": 1,
                "cdkType": "day",
                "num": 3
            },
            {
                "inviterCount": 3,
                "cdkType": "day",
                "num": 15
            },
            {
                "inviterCount": 5,
                "cdkType": "day",
                "num": 30
            }
        ]
    })
}

fn entry_config_payload() -> JsonValue {
    json!({
        "client": {
            "support": "--"
        },
        "business": {
            "cooperation": "--"
        },
        "launchModal": {
            "id": "nebula-campaign-202402",
            "enabled": true,
            "title": "新用户福利",
            "subtitle": "注册即送 3 天加速时长",
            "description": "新用户注册即享 3 天免费加速特权，立即开启流畅网络体验！",
            "image": "https://cdn.jsdelivr.net/gh/zerounary/cdn-assets/nebula/launch-modal.png",
            "accent": "linear-gradient(135deg, #ff8a5c, #7c3aed)",
            "dismissible": true,
            "actions": [
                {
                    "label": "立即注册",
                    "variant": "primary"
                },
                {
                    "label": "稍后再说",
                    "variant": "ghost",
                    "autoClose": true
                }
            ]
        }
    })
}

async fn seed_accounts(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let count = account_user::Entity::find().count(db).await?;
    if count == 0 {
        let password = hash_password("secret");
        account_user::ActiveModel {
            id: Set("9777888".to_string()),
            phone: Set("13800001234".to_string()),
            password_hash: Set(password),
            name: Set("User".to_string()),
            valid_until: Set((Utc::now() + Duration::days(365)).into()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn hash_password(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn seed_admin_config(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    // 初始化节点配置
    let node_id = 41u64;
    let node_config_exists = admin_node_config::Entity::find_by_id(node_id)
        .one(db)
        .await?;
    
    if let Some(existing) = node_config_exists {
        let mut active: admin_node_config::ActiveModel = existing.into();
        active.node_token = Set(Some("2a8fe744-1d46-43b7-9466-ebd46d058fb4".to_string()));
        active.node_shared_secret = Set(Some("306b0579-5cc2-4e21-8ff0-e4c2ad337c8e".to_string()));
        active.node_comm_mode = Set("grpc".to_string());
        let _ = active.update(db).await?;
        return Ok(());
    }

    if node_config_exists.is_none() {
        admin_node_config::ActiveModel {
            node_id: Set(node_id),
            node_type: Set("VlessReality".to_string()),
            node_speed_limit: Set(0),
            traffic_rate: Set(1.0),
            sort: Set(1),
            node_token: Set(Some("2a8fe744-1d46-43b7-9466-ebd46d058fb4".to_string())),
            node_shared_secret: Set(Some("306b0579-5cc2-4e21-8ff0-e4c2ad337c8e".to_string())),
            node_comm_mode: Set("grpc".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        admin_inbound::ActiveModel {
            node_id: Set(node_id),
            tag: Set("entrydoor".to_string()),
            protocol: Set("vless".to_string()),
            port: Set(10086),
            listen: Set(None),
            settings: Set(json!({
                "decryption": "none"
            })),
            stream_settings: Set(Some(json!({
                "network": "tcp",
                "security": "reality",
                "realitySettings": {
                    "show": false,
                    "dest": "www.cloudflare.com:443",
                    "xver": 0,
                    "serverNames": ["www.cloudflare.com"],
                    "privateKey": "aNd7UkHbEak7xUDUAcUCycsbi8sjk71TfNB5ZOp0yUk",
                    "shortIds": ["a66cafe7"],

                    "serverName": "www.cloudflare.com",
                    "publicKey": "JbPBpbjEHQiL87HpoJ6wZ3o9wSTyjzTIN9ysP6tnywU",
                    "shortId": "a66cafe7",
                    "fingerprint": "chrome",
                    "spiderX": "/"
                }
            }))),
            sniffing: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await?;
        
        // 初始化用户
        let users = vec![
            (1u64, "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(), 5u64, 0u64),
            (2u64, "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(), 1u64, 0u64),
            (3u64, "c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(), 1u64, 0u64),
            (4u64, "d4e5f6a7-b8c9-0123-def0-234567890123".to_string(), 1u64, 0u64),
            (5u64, "e5f6a7b8-c9d0-1234-ef01-345678901234".to_string(), 1u64, 0u64),
            (6u64, "f6a7b8c9-d0e1-2345-f012-456789012345".to_string(), 1u64, 0u64),
        ];
        
        for (id, uuid, st, dt) in users {
            admin_user::ActiveModel {
                id: Set(id),
                node_id: Set(node_id),
                uuid: Set(uuid),
                st: Set(st),
                dt: Set(dt),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        
        // 初始化上游代理
        let outbounds = vec![
            (
                "block".to_string(),
                "blackhole".to_string(),
                json!({
                    "response": {
                        "type": "http"
                    }
                }),
                None,
            ),
            (
                "direct".to_string(),
                "freedom".to_string(),
                json!({}),
                None,
            ),
            (
                "ss_1".to_string(),
                "shadowsocks".to_string(),
                json!({
                    "servers": [{
                        "address": "112.164.191.18",
                        "port": 6303,
                        "method": "aes-256-gcm",
                        "password": "na71"
                    }]
                }),
                None,
            ),
            (
                "ss_2".to_string(),
                "shadowsocks".to_string(),
                json!({
                    "servers": [{
                        "address": "118.40.250.41",
                        "port": 6304,
                        "method": "aes-256-gcm",
                        "password": "na71"
                    }]
                }),
                None,
            ),
            (
                "ss_3".to_string(),
                "shadowsocks".to_string(),
                json!({
                    "servers": [{
                        "address": "119.200.207.26",
                        "port": 6305,
                        "method": "aes-256-gcm",
                        "password": "na71"
                    }]
                }),
                None,
            ),
            (
                "ss_4".to_string(),
                "shadowsocks".to_string(),
                json!({
                    "servers": [{
                        "address": "121.149.218.164",
                        "port": 6306,
                        "method": "aes-256-gcm",
                        "password": "na71"
                    }]
                }),
                None,
            ),
            (
                "ss_5".to_string(),
                "shadowsocks".to_string(),
                json!({
                    "servers": [{
                        "address": "121.179.252.144",
                        "port": 6307,
                        "method": "aes-256-gcm",
                        "password": "na71"
                    }]
                }),
                None,
            ),
        ];
        
        for (tag, protocol, settings, stream_settings) in outbounds {
            admin_outbound::ActiveModel {
                node_id: Set(node_id),
                tag: Set(tag),
                protocol: Set(protocol),
                settings: Set(settings),
                send_through: Set(None),
                stream_settings: Set(stream_settings),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        
        // 初始化用户映射
        let mappings = vec![
            ("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(), "ss_1".to_string()),
            ("b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(), "ss_2".to_string()),
            ("c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(), "ss_3".to_string()),
            ("d4e5f6a7-b8c9-0123-def0-234567890123".to_string(), "ss_4".to_string()),
            ("e5f6a7b8-c9d0-1234-ef01-345678901234".to_string(), "ss_5".to_string()),
        ];
        
        for (uuid, outbound_tag) in mappings {
            admin_user_mapping::ActiveModel {
                node_id: Set(node_id),
                uuid: Set(uuid),
                outbound_tag: Set(outbound_tag),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        
        // 初始化路由配置
        admin_routing::ActiveModel {
            node_id: Set(node_id),
            domain_strategy: Set("AsIs".to_string()),
            rules: Set(json!([])),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    
    Ok(())
}
