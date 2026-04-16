use crate::parse::parse_progress_line;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
}

pub async fn fetch_metadata(app: &AppHandle, url: &str) -> Result<Metadata> {
    let (mut rx, _child) = app
        .shell()
        .sidecar("yt-dlp")?
        .args(["--dump-json", "--no-playlist", "--skip-download", url])
        .spawn()
        .context("spawn yt-dlp --dump-json")?;

    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();
    let mut exit_code: Option<i32> = None;
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(b) => stdout_buf.push_str(&String::from_utf8_lossy(&b)),
            CommandEvent::Stderr(b) => stderr_buf.push_str(&String::from_utf8_lossy(&b)),
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
            }
            _ => {}
        }
    }
    if exit_code != Some(0) {
        let tail: String = stderr_buf.lines().last().unwrap_or("").into();
        return Err(anyhow!(if tail.is_empty() {
            "yt-dlp failed".into()
        } else {
            tail
        }));
    }
    let first_line = stdout_buf.lines().next().unwrap_or("{}");
    let meta: Metadata = serde_json::from_str(first_line).context("parse dump-json")?;
    Ok(meta)
}

pub async fn run_download<F: FnMut(f64)>(
    app: &AppHandle,
    url: &str,
    download_dir: &str,
    ffmpeg_path: &str,
    mut on_progress: F,
) -> Result<Option<PathBuf>> {
    let output_template = format!("{}/%(title)s.%(ext)s", download_dir.trim_end_matches('/'));
    let (mut rx, _child) = app
        .shell()
        .sidecar("yt-dlp")?
        .args([
            url,
            "-f",
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "--merge-output-format",
            "mp4",
            "--no-playlist",
            "--replace-in-metadata",
            "title",
            " ",
            "_",
            "--ffmpeg-location",
            ffmpeg_path,
            "--newline",
            "--progress-template",
            "download:%(progress._percent)s",
            "--print",
            "after_move:completed:%(filepath)s",
            "-o",
            &output_template,
        ])
        .spawn()
        .context("spawn yt-dlp")?;

    let mut completed_path: Option<PathBuf> = None;
    let mut stderr_buf = String::new();
    let mut exit_code: Option<i32> = None;
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(b) => {
                let chunk = String::from_utf8_lossy(&b);
                for line in chunk.lines() {
                    if let Some(p) = parse_progress_line(line) {
                        on_progress(p);
                    } else if let Some(rest) = line.strip_prefix("completed:") {
                        completed_path = Some(PathBuf::from(rest.trim()));
                    }
                }
            }
            CommandEvent::Stderr(b) => stderr_buf.push_str(&String::from_utf8_lossy(&b)),
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
            }
            _ => {}
        }
    }
    if exit_code != Some(0) {
        let tail = stderr_buf.lines().last().unwrap_or("").to_string();
        return Err(anyhow!(if tail.is_empty() {
            "download failed".into()
        } else {
            tail
        }));
    }
    Ok(completed_path)
}

pub fn ffmpeg_sidecar_path(app: &AppHandle) -> Result<String> {
    let _ = app;
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| anyhow!("no exe dir"))?;
    let triple = current_target_triple();

    // Production: Tauri bundles sidecars without the triple suffix
    #[cfg(target_os = "windows")]
    let bundled = "ffmpeg.exe";
    #[cfg(not(target_os = "windows"))]
    let bundled = "ffmpeg";
    let candidate = dir.join(bundled);
    if candidate.exists() {
        return Ok(candidate.to_string_lossy().to_string());
    }

    // Dev: src-tauri/binaries/ffmpeg-{triple}
    #[cfg(target_os = "windows")]
    let dev_name = format!("ffmpeg-{triple}.exe");
    #[cfg(not(target_os = "windows"))]
    let dev_name = format!("ffmpeg-{triple}");
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(&dev_name);
    if dev.exists() {
        return Ok(dev.to_string_lossy().to_string());
    }
    Err(anyhow!("ffmpeg sidecar not found"))
}

fn current_target_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-msvc"
    }
}
