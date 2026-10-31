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
    /// 快捷方式附带的命令行参数（原样保留，启动时传给目标程序）
    pub args: Option<String>,
    pub category: Option<String>,
    pub open_count: Option<u64>,
    pub last_opened: Option<i64>,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub notes: Option<String>,
    pub is_pinned: Option<bool>,
    pub dir_size_calculated: Option<bool>,
    pub is_reparse_point: Option<bool>,
}

/// 批量图标结果：资源管理器一次取当前可见项的图标。
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIcons {
    /// 入参 path -> data URL（取不到图标时为空串）
    pub icons: std::collections::HashMap<String, String>,
    /// 可跨文件复用的「通用图标」：键为 `dir:{尺寸}` / `ext:{尺寸}:{扩展名}`，
    /// 前端按同一规则即可直接命中，不必等每个文件的图标返回。
    pub generic: std::collections::HashMap<String, String>,
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
    #[serde(alias = "current_category")]
    pub current_category: String,
    #[serde(alias = "sort_method")]
    pub sort_method: String,
    #[serde(alias = "sort_order")]
    pub sort_order: String,
    #[serde(alias = "classify_method")]
    pub classify_method: String,
    #[serde(alias = "explorer_path")]
    pub explorer_path: Option<String>,
}
