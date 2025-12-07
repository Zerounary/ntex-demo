use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardPayload {
    pub announcements: Vec<Announcement>,
    pub hero_stats: Vec<StatItem>,
    pub quick_panels: Vec<QuickPanel>,
    pub featured_snapshots: Vec<Snapshot>,
    pub marquee_items: Vec<LabelValue>,
    pub hologram_glyphs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub id: i32,
    pub title: String,
    pub date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatItem {
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickPanel {
    pub title: String,
    pub desc: String,
    pub icon: String,
    pub accent: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub label: String,
    pub value: String,
    pub signal: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelValue {
    pub label: String,
    #[serde(alias = "detail")]
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPayload {
    pub categories: Vec<String>,
    pub curated_collections: Vec<CollectionItem>,
    pub ops_memos: Vec<LabelValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItem {
    pub title: String,
    pub desc: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsMeta {
    pub version: String,
    pub status_label: String,
    pub status_description: String,
    pub hero_stats: Vec<StatItem>,
    pub regions: Vec<RegionOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationConfig {
    pub menu: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationItem {
    pub name: String,
    pub path: String,
    pub icon: String,
}
