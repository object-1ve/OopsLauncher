use serde::{Deserialize, Serialize};

// 文件信息结构体
#[derive(Debug, Serialize, Deserialize)]
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
}

// 分类结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
}
