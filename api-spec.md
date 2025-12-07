# API Contract (Mock-backed)

This project now centralizes every frontend data request in `src/api/`. Each helper mimics the eventual backend REST endpoint and returns the same JSON shape. Backends can implement the routes below without being coupled to the Vue/Tauri code.

## 1. Accelerator Bootstrap

- **Endpoint:** `GET /api/accelerator/bootstrap`
- **Used by:** `src/stores/accelerator.ts`
- **Purpose:** Load the full accelerator snapshot (games, node profiles, optional signed-in user).
- **Response**

```jsonc
{
  "games": [
    { "id": "7", "name": "Chrome 加速测试", "icon": "i-mdi-google-chrome", "status": "idle", "ping": 0 }
    // ...
  ],
  "profiles": [
    {
      "id": "node-chrome-test",
      "gameId": "7",
      "displayName": "Chrome · 调试隧道",
      "processName": "chrome.exe",
      "vmessUuid": "9acea125-3ca7-1212-2121-000000010135",
      "vmessServer": "123.206.203.43",
      "vmessPort": 11111,
      "vmessEmail": "lol-kr@acc.local",
      "udpProxy": "123.206.203.43:10810",
      "mode": "进程模式",
      "status": "空闲",
      "region": "测试",
      "ping": 5
    }
  ],
  "user": {
    "id": "9777888",
    "name": "User",
    "validUntil": "2025/12/31"
  }
}
```

- **Side channel:** After fetching, the frontend calls the Tauri command `sync_game_profiles` with the `profiles` payload so the Rust backend mirrors the same catalog.

## 2. Dashboard Overview

- **Endpoint:** `GET /api/dashboard`
- **Used by:** `src/pages/index.vue`
- **Response**

```jsonc
{
  "announcements": [{ "id": 1, "title": "【维护】2月全球节点例行维护", "date": "12:30" }],
  "heroStats": [{ "label": "在线节点", "value": "128", "hint": "+12 新增" }],
  "quickPanels": [{ "title": "量子加速", "desc": "独占物理专线", "icon": "i-mdi-flash", "accent": "rgba(...)" }],
  "featuredSnapshots": [{ "label": "亚服 · 旗舰节点", "value": "B1-东京 4001", "signal": "低负载" }],
  "marqueeItems": [{ "label": "NEBULA CORE", "value": "SYNC 99.98%" }],
  "hologramGlyphs": ["Δ", "Ω", "Ξ", "⌁", "Φ", "∑"]
}
```

## 3. Library Content

- **Endpoint:** `GET /api/library`
- **Used by:** `src/pages/library.vue`
- **Response**

```jsonc
{
  "categories": ["最新上线", "全部", "热门", "限免", "Steam", "橘子", "暴雪", "Epic"],
  "curatedCollections": [
    { "title": "对战优选", "desc": "FPS/竞技类低延迟专线", "value": "12 条" }
  ],
  "opsMemos": [
    { "label": "节点巡航", "detail": "亚洲集群调度完成" }
  ]
}
```

## 4. Settings Metadata

- **Endpoint:** `GET /api/settings/meta`
- **Used by:** `src/pages/settings.vue`
- **Response**

```jsonc
{
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
}
```

## 5. Navigation Configuration

- **Endpoint:** `GET /api/navigation`
- **Used by:** `src/components/Sidebar.vue`
- **Response**

```jsonc
{
  "menu": [
    { "name": "首页", "path": "/", "icon": "i-mdi-home-variant-outline" },
    { "name": "我的加速", "path": "/my-boosts", "icon": "i-mdi-rocket-launch-outline" },
    { "name": "游戏库", "path": "/library", "icon": "i-mdi-gamepad-variant-outline" }
  ]
}
```

## 6. Accelerator Profile Sync (Desktop bridge)

- **Endpoint:** `POST /api/accelerator/profiles`
- **Used by:** Tauri command `sync_game_profiles` (`src-tauri/src/lib.rs`)
- **Request**

```jsonc
{
  "profiles": [ /* same shape as bootstrap -> profiles[] */ ]
}
```

- **Purpose:** Allows the UI/bootstrap to push the latest VMess/netfilter configuration down to the Rust core. Backend services that already persist these profiles can use a regular POST endpoint; the desktop app reuses the same payload.

## 7. 微信扫码登录

- **Endpoints:**
  - `POST /api/auth/wechat/ticket` → returns a QR ticket
  - `GET /api/auth/wechat/status?ticketId=xxx` → poll login status
- **Request (ticket)**

```jsonc
{ "scene": "nebula-login-desktop" }
```

- **Ticket Response**

```jsonc
{
  "ticketId": "mock-ticket-001",
  "qrCodeUrl": "https://example.com/mock-qrcode.png",
  "expiresIn": 120,
  "status": "pending",
  "scene": "nebula-login-desktop"
}
```

- **Status Response**

```jsonc
{
  "success": true,
  "ticketId": "mock-ticket-001",
  "status": "confirmed",
  "user": {
    "id": "9777888",
    "name": "User",
    "validUntil": "2025/12/31"
  }
}
```

- **Notes**
  - `status` transitions: `pending` → `scanned` → `confirmed` or `expired`.
  - Once `status=confirmed` with `success=true`, the frontend emits `login-success` and reuses the `user` payload to hydrate the store.
  - On `expired`, the frontend automatically re-requests a fresh ticket.

---

> **Mocking strategy:** Each helper in `src/api/` calls `simulateNetworkLatency()` and returns a clone of the mock dataset from `src/api/mockData.ts`. When the real backend is ready, replace the helper implementation with an HTTP request (e.g., `fetch`/`axios`) while keeping the response interface identical. This guarantees the UI and the Tauri backend continue to work with production data without further code changes.
- **Account Login**
  - `POST /api/auth/account`

```jsonc
{
  "phone": "13800001234",
  "password": "secret",
  "remember": true
}
```

```jsonc
{
  "success": true,
  "token": "mock-token-abc123",
  "user": {
    "id": "9777888",
    "name": "User",
    "validUntil": "2025/12/31"
  }
}
```

