use crate::{db::Db, model::Download, queue, settings::Settings, ytdlp};
use rusqlite::params;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct StartedDownload {
    pub id: i64,
}

#[tauri::command]
pub async fn start_download(app: AppHandle, url: String) -> Result<StartedDownload, String> {
    let meta = ytdlp::fetch_metadata(&app, &url)
        .await
        .map_err(|e| e.to_string())?;
    let db = app.state::<Db>();
    let id = {
        let conn = db.0.lock().unwrap();
        conn.execute(
            "INSERT INTO downloads(url,title,thumbnail,status,created_at) VALUES (?,?,?,'queued',?)",
            params![url, meta.title, meta.thumbnail, now_millis()],
        )
        .map_err(|e| e.to_string())?;
        conn.last_insert_rowid()
    };
    let _ = app.emit("downloads_changed", ());
    queue::notify(&app);
    Ok(StartedDownload { id })
}

#[tauri::command]
pub fn retry_download(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Db>();
    db.0.lock()
        .unwrap()
        .execute(
            "UPDATE downloads SET status='queued', progress=0, error=NULL, completed_at=NULL WHERE id=?",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    let _ = app.emit("downloads_changed", ());
    queue::notify(&app);
    Ok(())
}

#[tauri::command]
pub fn remove_from_queue(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Db>();
    db.0.lock()
        .unwrap()
        .execute(
            "DELETE FROM downloads WHERE id=? AND status='queued'",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    let _ = app.emit("downloads_changed", ());
    Ok(())
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    let db = app.state::<Db>();
    db.0.lock()
        .unwrap()
        .execute(
            "DELETE FROM downloads WHERE status IN ('completed','failed')",
            [],
        )
        .map_err(|e| e.to_string())?;
    let _ = app.emit("downloads_changed", ());
    Ok(())
}

#[tauri::command]
pub fn list_downloads(app: AppHandle) -> Result<Vec<Download>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id,url,title,thumbnail,file_path,status,progress,error,created_at,completed_at \
             FROM downloads ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Download::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Settings::load_or_init(&app_data_dir(&app)?).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_download_dir(app: AppHandle, dir: String) -> Result<Settings, String> {
    let dir_buf = PathBuf::from(&dir);
    if !dir_buf.is_dir() {
        return Err("선택한 경로가 폴더가 아닙니다".into());
    }
    let dir_path = app_data_dir(&app)?;
    let mut s = Settings::load_or_init(&dir_path).map_err(|e| e.to_string())?;
    s.download_dir = dir_buf;
    s.save(&dir_path).map_err(|e| e.to_string())?;
    Ok(s)
}
