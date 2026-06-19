use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::db;
use crate::error::AppError;
use crate::services::file_service;

const FOLDER_SIZE_TASK_EVENT: &str = "folder-size-task-item";
const FOLDER_SIZE_TASK_COMPLETE_EVENT: &str = "folder-size-task-complete";
const FOLDER_SIZE_CONCURRENCY_LIMIT: usize = 5;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderSizeTaskItemPayload {
    task_id: String,
    path: String,
    size: Option<u64>,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderSizeTaskCompletePayload {
    task_id: String,
}

/// Starts a background task that calculates folder sizes and emits item updates.
pub fn start_folder_size_task(app: AppHandle, paths: Vec<String>) -> Result<String, AppError> {
    if paths.is_empty() {
        return Err(AppError::Other("没有可计算的文件夹".into()));
    }

    let task_id = generate_task_id();
    let app_handle = app.clone();
    let worker_paths = paths;
    let worker_task_id = task_id.clone();

    thread::spawn(move || run_folder_size_task(app_handle, worker_task_id, worker_paths));

    Ok(task_id)
}

fn run_folder_size_task(app: AppHandle, task_id: String, paths: Vec<String>) {
    let worker_count = paths.len().min(FOLDER_SIZE_CONCURRENCY_LIMIT);
    let queue = Arc::new(Mutex::new(VecDeque::from(paths)));
    let mut handles = Vec::with_capacity(worker_count);

    for _ in 0..worker_count {
        let app_handle = app.clone();
        let task_id_clone = task_id.clone();
        let queue_clone = Arc::clone(&queue);

        handles.push(thread::spawn(move || loop {
            let next_path = match queue_clone.lock() {
                Ok(mut guard) => guard.pop_front(),
                Err(_) => None,
            };

            let Some(path) = next_path else {
                break;
            };

            match file_service::calculate_dir_size_single(path.clone()) {
                Ok(size) => {
                    // Save to database
                    if let Ok(conn) = db::get_db_connection(&app_handle) {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO folder_sizes (path, size, last_updated) VALUES (?1, ?2, ?3)",
                            rusqlite::params![path, size, now],
                        );
                    }

                    let _ = app_handle.emit(
                        FOLDER_SIZE_TASK_EVENT,
                        FolderSizeTaskItemPayload {
                            task_id: task_id_clone.clone(),
                            path,
                            size: Some(size),
                            error: None,
                        },
                    );
                }
                Err(error) => {
                    let _ = app_handle.emit(
                        FOLDER_SIZE_TASK_EVENT,
                        FolderSizeTaskItemPayload {
                            task_id: task_id_clone.clone(),
                            path,
                            size: None,
                            error: Some(error.to_string()),
                        },
                    );
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    let _ = app.emit(
        FOLDER_SIZE_TASK_COMPLETE_EVENT,
        FolderSizeTaskCompletePayload { task_id },
    );
}

fn generate_task_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("folder-size-{:x}", nanos)
}
