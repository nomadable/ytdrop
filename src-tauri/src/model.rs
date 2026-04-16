use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Completed,
    Failed,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl ToSql for DownloadStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for DownloadStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(match s {
            "queued" => Self::Queued,
            "downloading" => Self::Downloading,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            other => return Err(FromSqlError::Other(format!("bad status: {other}").into())),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub status: DownloadStatus,
    pub progress: f64,
    pub error: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<i64>,
}

impl Download {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get("id")?,
            url: r.get("url")?,
            title: r.get("title")?,
            thumbnail: r.get("thumbnail")?,
            file_path: r.get("file_path")?,
            status: r.get("status")?,
            progress: r.get("progress")?,
            error: r.get("error")?,
            created_at: r.get("created_at")?,
            completed_at: r.get("completed_at")?,
        })
    }
}
