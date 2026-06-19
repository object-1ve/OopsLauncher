use serde::{Deserialize, Serialize};

// 文件信息结构体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub path: String,
    pub size: u64,
    pub r#type: String,
    pub icon: String,
    pub content: Option<String>,
    pub category: Option<String>,
    pub open_count: Option<u64>,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub notes: Option<String>,
    pub is_pinned: Option<bool>,
    pub dir_size_calculated: Option<bool>,
    pub is_reparse_point: Option<bool>,
}

// 分类结构体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherState {
    pub current_category: String,
    pub sort_method: String,
    pub sort_order: String,
    pub classify_method: String,
    pub explorer_path: Option<String>,
}
