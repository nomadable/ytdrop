use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "downloadDir")]
    pub download_dir: PathBuf,
}

impl Settings {
    pub fn default_for_platform() -> Self {
        let dir = dirs::download_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join("Downloads"))
                .unwrap_or_else(|| PathBuf::from("."))
        });
        Self { download_dir: dir }
    }

    pub fn load_or_init(app_dir: &Path) -> Result<Self> {
        let path = app_dir.join("settings.json");
        if path.exists() {
            let s = std::fs::read_to_string(&path).context("read settings")?;
            Ok(serde_json::from_str(&s)?)
        } else {
            let s = Self::default_for_platform();
            s.save(app_dir)?;
            Ok(s)
        }
    }

    pub fn save(&self, app_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(app_dir)?;
        let path = app_dir.join("settings.json");
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_then_load_roundtrip() {
        let dir = tempdir().unwrap();
        let s1 = Settings::load_or_init(dir.path()).unwrap();
        let s2 = Settings::load_or_init(dir.path()).unwrap();
        assert_eq!(s1.download_dir, s2.download_dir);
    }

    #[test]
    fn save_persists_change() {
        let dir = tempdir().unwrap();
        let mut s = Settings::load_or_init(dir.path()).unwrap();
        s.download_dir = PathBuf::from("/tmp/custom");
        s.save(dir.path()).unwrap();
        let loaded = Settings::load_or_init(dir.path()).unwrap();
        assert_eq!(loaded.download_dir, PathBuf::from("/tmp/custom"));
    }
}
