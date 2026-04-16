use crate::{db::Db, model::DownloadStatus, settings::Settings, ytdlp};
use anyhow::Result;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

pub struct QueueState {
    pub notify: Arc<Notify>,
}

#[derive(Serialize, Clone)]
struct ProgressEvent {
    id: i64,
    progress: f64,
}

pub fn spawn_worker(app: AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let next = pick_next_queued(&handle);
            match next {
                Ok(Some((id, url))) => {
                    if let Err(e) = process_one(&handle, id, &url).await {
                        eprintln!("download {} failed: {:#}", id, e);
                        mark_failed(&handle, id, &e.to_string());
                        emit_list_changed(&handle);
                    }
                }
                Ok(None) => {
                    let notify = handle.state::<QueueState>().notify.clone();
                    notify.notified().await;
                }
                Err(e) => {
                    eprintln!("queue pick error: {:#}", e);
                }
            }
        }
    });
}

fn pick_next_queued(app: &AppHandle) -> Result<Option<(i64, String)>> {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, url FROM downloads WHERE status='queued' ORDER BY created_at ASC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(r) = rows.next()? {
        Ok(Some((r.get(0)?, r.get(1)?)))
    } else {
        Ok(None)
    }
}

async fn process_one(app: &AppHandle, id: i64, url: &str) -> Result<()> {
    {
        let db = app.state::<Db>();
        db.0.lock().unwrap().execute(
            "UPDATE downloads SET status='downloading', progress=0, error=NULL WHERE id=?",
            params![id],
        )?;
    }
    emit_list_changed(app);

    let ffmpeg = ytdlp::ffmpeg_sidecar_path(app)?;
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let settings = Settings::load_or_init(&app_dir)?;
    let dir_str = settings.download_dir.to_string_lossy().to_string();

    let app_for_cb = app.clone();
    let result = ytdlp::run_download(app, url, &dir_str, &ffmpeg, move |p| {
        persist_progress(&app_for_cb, id, p);
        let _ = app_for_cb.emit("download_update", ProgressEvent { id, progress: p });
    })
    .await;

    match result {
        Ok(file_path) => {
            let now = now_millis();
            let db = app.state::<Db>();
            db.0.lock().unwrap().execute(
                "UPDATE downloads SET status='completed', progress=1.0, file_path=?, completed_at=? WHERE id=?",
                params![file_path.as_ref().map(|p| p.to_string_lossy().to_string()), now, id],
            )?;
            emit_list_changed(app);
            use tauri_plugin_notification::NotificationExt;
            let title = fetch_title(app, id).unwrap_or_else(|| "Download complete".into());
            let _ = app
                .notification()
                .builder()
                .title("ytdrop")
                .body(format!("완료: {title}"))
                .show();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn persist_progress(app: &AppHandle, id: i64, p: f64) {
    if let Ok(conn) = app.state::<Db>().0.lock() {
        let _ = conn.execute(
            "UPDATE downloads SET progress=? WHERE id=? AND status='downloading'",
            params![p, id],
        );
    }
}

fn mark_failed(app: &AppHandle, id: i64, msg: &str) {
    if let Ok(conn) = app.state::<Db>().0.lock() {
        let _ = conn.execute(
            "UPDATE downloads SET status='failed', error=? WHERE id=?",
            params![msg, id],
        );
    }
}

fn fetch_title(app: &AppHandle, id: i64) -> Option<String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().ok()?;
    conn.query_row(
        "SELECT title FROM downloads WHERE id=?",
        params![id],
        |r| r.get::<_, Option<String>>(0),
    )
    .ok()?
}

fn emit_list_changed(app: &AppHandle) {
    let _ = app.emit("downloads_changed", ());
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn notify(app: &AppHandle) {
    let _ = app.state::<QueueState>().notify.notify_one();
}

// Silence unused import warning
#[allow(dead_code)]
fn _types(_s: DownloadStatus) {}
