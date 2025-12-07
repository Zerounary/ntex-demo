use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, JsonValue, PaginatorTrait, Set};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::infrastructure::persistence::{
    accelerator_game, accelerator_profile, accelerator_user, account_user, config_entry,
};

pub async fn seed(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    seed_accelerator(db).await?;
    seed_config_entries(db).await?;
    seed_accounts(db).await?;
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
        }];

        for game in games {
            game.insert(db).await?;
        }
    }

    let profile_count = accelerator_profile::Entity::find().count(db).await?;
    if profile_count == 0 {
        let profiles = vec![accelerator_profile::ActiveModel {
            id: Set("node-chrome-test".to_string()),
            game_id: Set("7".to_string()),
            display_name: Set("Chrome · 调试隧道".to_string()),
            process_name: Set("chrome.exe".to_string()),
            vmess_uuid: Set("9acea125-3ca7-1212-2121-000000010135".to_string()),
            vmess_server: Set("123.206.203.43".to_string()),
            vmess_port: Set(11111),
            vmess_email: Set("lol-kr@acc.local".to_string()),
            udp_proxy: Set("123.206.203.43:10810".to_string()),
            mode: Set("进程模式".to_string()),
            status: Set("空闲".to_string()),
            region: Set("测试".to_string()),
            ping: Set(5),
        }];

        for profile in profiles {
            profile.insert(db).await?;
        }
    }

    let user_count = accelerator_user::Entity::find().count(db).await?;
    if user_count == 0 {
        accelerator_user::ActiveModel {
            id: Set("9777888".to_string()),
            name: Set("User".to_string()),
            valid_until: Set((Utc::now() + Duration::days(365)).into()),
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
    ];

    for (key, payload) in entries {
        config_entry::ActiveModel {
            key: Set(key.to_string()),
            payload: Set(payload),
            updated_at: Set(Utc::now().into()),
        }
        .save(db)
        .await?;
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
