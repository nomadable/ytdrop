# ytdrop

Cross-platform yt-dlp desktop app. Paste a URL, get an mp4.

## Install (macOS)
1. Download `ytdrop_x.y.z_aarch64.dmg` (Apple Silicon) or `_x64.dmg` (Intel).
2. Open the DMG, drag `ytdrop.app` to Applications.
3. First launch: right-click → Open → Open (Gatekeeper).

## Install (Windows)
1. Download `ytdrop_x.y.z_x64_en-US.msi`.
2. Run it. SmartScreen: "More info" → "Run anyway".

## Dev
```bash
pnpm install
pnpm fetch-binaries
pnpm tauri dev
```

## Build
```bash
pnpm tauri build
```

## License
MIT
