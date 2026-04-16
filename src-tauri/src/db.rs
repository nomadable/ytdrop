use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

pub fn open(app_dir: &PathBuf) -> Result<Db> {
    std::fs::create_dir_all(app_dir).context("create app dir")?;
    let conn = Connection::open(app_dir.join("ytdrop.db"))?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS downloads (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            url          TEXT    NOT NULL,
            title        TEXT,
            thumbnail    TEXT,
            file_path    TEXT,
            status       TEXT    NOT NULL CHECK (status IN ('queued','downloading','completed','failed')),
            progress     REAL    NOT NULL DEFAULT 0,
            error        TEXT,
            created_at   INTEGER NOT NULL,
            completed_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_status_created ON downloads(status, created_at);

        -- Recover from crashes: any 'downloading' row on startup becomes 'queued'.
        UPDATE downloads SET status='queued', progress=0 WHERE status='downloading';
    "#,
    )?;
    Ok(Db(Mutex::new(conn)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use tempfile::tempdir;

    #[test]
    fn creates_schema_idempotently() {
        let dir = tempdir().unwrap();
        let p = dir.path().to_path_buf();
        let _ = open(&p).unwrap();
        let _ = open(&p).unwrap(); // second call should not fail
    }

    #[test]
    fn downloading_rows_recover_to_queued() {
        let dir = tempdir().unwrap();
        let p = dir.path().to_path_buf();
        {
            let db = open(&p).unwrap();
            db.0.lock()
                .unwrap()
                .execute(
                    "INSERT INTO downloads(url,status,created_at) VALUES (?,?,?)",
                    params!["https://x", "downloading", 1i64],
                )
                .unwrap();
        }
        let db = open(&p).unwrap();
        let status: String = db
            .0
            .lock()
            .unwrap()
            .query_row(
                "SELECT status FROM downloads WHERE url='https://x'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "queued");
    }
}
