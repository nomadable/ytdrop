#!/usr/bin/env node
// Downloads yt-dlp + ffmpeg standalone builds into src-tauri/binaries/
// using Tauri's {name}-{triple}{ext} naming so they can be registered as sidecars.
//
// Usage:
//   pnpm fetch-binaries                    # fetch all targets
//   pnpm fetch-binaries -- --target x86_64-pc-windows-msvc  # single target
//
// Sources:
//  - yt-dlp: https://github.com/yt-dlp/yt-dlp/releases/latest/download/
//  - ffmpeg (macOS): osxexperts.net (arm64), evermeet.cx (intel)
//  - ffmpeg (Windows): https://www.gyan.dev/ffmpeg/builds/

import { createWriteStream, existsSync, mkdirSync, chmodSync, readdirSync, copyFileSync } from "node:fs";
import { mkdir, rm } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import os from "node:os";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const OUT = path.join(ROOT, "src-tauri", "binaries");
mkdirSync(OUT, { recursive: true });

const isWin = os.platform() === "win32";

// Parse --target flag
const targetArg = process.argv.find((_, i, a) => a[i - 1] === "--target");
const ALL_TRIPLES = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"];
const targets = targetArg ? [targetArg] : ALL_TRIPLES;

async function download(url, dest) {
  console.log(`→ ${url}`);
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
  await pipeline(Readable.fromWeb(res.body), createWriteStream(dest));
}

function extractZip(zipPath, destDir) {
  if (isWin) {
    execFileSync("powershell", [
      "-NoProfile", "-Command",
      `Expand-Archive -Force -Path '${zipPath}' -DestinationPath '${destDir}'`
    ], { stdio: "inherit" });
  } else {
    execFileSync("unzip", ["-o", zipPath, "-d", destDir], { stdio: "inherit" });
  }
}

// yt-dlp_macos is universal2 (arm64 + x86_64).
const ytdlpTargets = {
  "aarch64-apple-darwin": { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos", ext: "" },
  "x86_64-apple-darwin":  { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos", ext: "" },
  "x86_64-pc-windows-msvc": { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe", ext: ".exe" },
};

async function fetchYtdlp(triple) {
  const { url, ext } = ytdlpTargets[triple];
  const dest = path.join(OUT, `yt-dlp-${triple}${ext}`);
  if (existsSync(dest)) { console.log(`✓ exists ${path.basename(dest)}`); return; }
  await download(url, dest);
  if (!isWin && !ext) chmodSync(dest, 0o755);
  console.log(`✓ ${path.basename(dest)}`);
}

async function fetchFfmpegMac(triple, arch) {
  const dest = path.join(OUT, `ffmpeg-${triple}`);
  if (existsSync(dest)) { console.log(`✓ exists ${path.basename(dest)}`); return; }
  const zipUrl = arch === "arm"
    ? "https://www.osxexperts.net/ffmpeg711arm.zip"
    : "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip";
  const tmpZip = path.join(os.tmpdir(), `ffmpeg-${triple}.zip`);
  await download(zipUrl, tmpZip);
  const extractDir = path.join(os.tmpdir(), `ffmpeg-${triple}-ext`);
  await rm(extractDir, { recursive: true, force: true });
  await mkdir(extractDir, { recursive: true });
  extractZip(tmpZip, extractDir);
  const bin = path.join(extractDir, "ffmpeg");
  copyFileSync(bin, dest);
  chmodSync(dest, 0o755);
  console.log(`✓ ${path.basename(dest)}`);
}

async function fetchFfmpegWin() {
  const triple = "x86_64-pc-windows-msvc";
  const dest = path.join(OUT, `ffmpeg-${triple}.exe`);
  if (existsSync(dest)) { console.log(`✓ exists ${path.basename(dest)}`); return; }
  const zipUrl = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
  const tmpZip = path.join(os.tmpdir(), "ffmpeg-win.zip");
  await download(zipUrl, tmpZip);
  const extractDir = path.join(os.tmpdir(), "ffmpeg-win-ext");
  await rm(extractDir, { recursive: true, force: true });
  await mkdir(extractDir, { recursive: true });
  extractZip(tmpZip, extractDir);
  const sub = readdirSync(extractDir).find(n => n.startsWith("ffmpeg-"));
  if (!sub) throw new Error("ffmpeg zip layout unexpected");
  const bin = path.join(extractDir, sub, "bin", "ffmpeg.exe");
  copyFileSync(bin, dest);
  console.log(`✓ ${path.basename(dest)}`);
}

async function main() {
  for (const triple of targets) {
    await fetchYtdlp(triple);
    if (triple === "aarch64-apple-darwin") await fetchFfmpegMac(triple, "arm");
    else if (triple === "x86_64-apple-darwin") await fetchFfmpegMac(triple, "");
    else if (triple === "x86_64-pc-windows-msvc") await fetchFfmpegWin();
  }
  console.log("\nSidecar binaries ready in src-tauri/binaries/");
}

main().catch(e => { console.error(e); process.exit(1); });
