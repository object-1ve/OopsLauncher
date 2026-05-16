use crate::error::AppError;
use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[derive(Deserialize)]
pub struct NotificationParams {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[tauri::command]
pub fn send_notification_custom(
    app: AppHandle,
    params: NotificationParams,
) -> Result<(), AppError> {
    let mut builder = app.notification().builder().title(&params.title).body(&params.body);

    if let Some(icon) = params.icon {
        if !icon.is_empty() {
            builder = builder.icon(&icon);
        }
    }

    builder.show().map_err(|e| AppError::Plugin(e.to_string()))?;

    Ok(())
}
