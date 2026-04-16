#!/usr/bin/env node
// Downloads yt-dlp + ffmpeg standalone builds into src-tauri/binaries/
// using Tauri's {name}-{triple}{ext} naming so they can be registered as sidecars.
//
// Sources:
//  - yt-dlp: https://github.com/yt-dlp/yt-dlp/releases/latest/download/
//  - ffmpeg (macOS): https://evermeet.cx/ffmpeg/ (arm64 / intel zips)
//  - ffmpeg (Windows): https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip
//
// Run: pnpm fetch-binaries

import { createWriteStream, existsSync, mkdirSync, chmodSync, readdirSync } from "node:fs";
import { mkdir, rm } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { execFileSync } from "node:child_process";
import path from "node:path";
import os from "node:os";

const ROOT = path.resolve(new URL(".", import.meta.url).pathname, "..");
const OUT = path.join(ROOT, "src-tauri", "binaries");
mkdirSync(OUT, { recursive: true });

// yt-dlp_macos is shipped as a universal2 binary covering arm64 + x86_64.
const ytdlpTargets = {
  "aarch64-apple-darwin": { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos", ext: "" },
  "x86_64-apple-darwin":  { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos", ext: "" },
  "x86_64-pc-windows-msvc": { url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe", ext: ".exe" },
};

async function download(url, dest) {
  console.log(`→ ${url}`);
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
  await pipeline(Readable.fromWeb(res.body), createWriteStream(dest));
}

async function fetchYtdlp() {
  for (const [triple, { url, ext }] of Object.entries(ytdlpTargets)) {
    const dest = path.join(OUT, `yt-dlp-${triple}${ext}`);
    if (existsSync(dest)) { console.log(`✓ exists ${path.basename(dest)}`); continue; }
    await download(url, dest);
    if (!ext) chmodSync(dest, 0o755);
    console.log(`✓ ${path.basename(dest)}`);
  }
}

async function fetchFfmpegMac(triple, arch) {
  const dest = path.join(OUT, `ffmpeg-${triple}`);
  if (existsSync(dest)) { console.log(`✓ exists ${path.basename(dest)}`); return; }
  // osxexperts.net ships arm64-native macOS builds; evermeet.cx ships Intel builds.
  const zipUrl = arch === "arm"
    ? "https://www.osxexperts.net/ffmpeg711arm.zip"
    : "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip";
  const tmpZip = path.join(os.tmpdir(), `ffmpeg-${triple}.zip`);
  await download(zipUrl, tmpZip);
  const extractDir = path.join(os.tmpdir(), `ffmpeg-${triple}-ext`);
  await rm(extractDir, { recursive: true, force: true });
  await mkdir(extractDir, { recursive: true });
  execFileSync("unzip", ["-o", tmpZip, "-d", extractDir], { stdio: "inherit" });
  const bin = path.join(extractDir, "ffmpeg");
  execFileSync("cp", [bin, dest]);
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
  execFileSync("unzip", ["-o", tmpZip, "-d", extractDir], { stdio: "inherit" });
  const sub = readdirSync(extractDir).find(n => n.startsWith("ffmpeg-"));
  if (!sub) throw new Error("ffmpeg zip layout unexpected");
  const bin = path.join(extractDir, sub, "bin", "ffmpeg.exe");
  execFileSync("cp", [bin, dest]);
  console.log(`✓ ${path.basename(dest)}`);
}

async function main() {
  await fetchYtdlp();
  await fetchFfmpegMac("aarch64-apple-darwin", "arm");
  await fetchFfmpegMac("x86_64-apple-darwin", "");
  await fetchFfmpegWin();
  console.log("\nAll sidecar binaries present in src-tauri/binaries/");
}

main().catch(e => { console.error(e); process.exit(1); });
