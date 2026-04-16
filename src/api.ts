import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Download, Settings } from "./types";

export const api = {
  startDownload: (url: string) =>
    invoke<{ id: number }>("start_download", { url }),
  retry: (id: number) => invoke<void>("retry_download", { id }),
  remove: (id: number) => invoke<void>("remove_from_queue", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  list: () => invoke<Download[]>("list_downloads"),
  getSettings: () => invoke<Settings>("get_settings"),
  setDownloadDir: (dir: string) =>
    invoke<Settings>("set_download_dir", { dir }),
};

export function onDownloadsChanged(cb: () => void): Promise<UnlistenFn> {
  return listen("downloads_changed", cb);
}

export function onProgress(
  cb: (e: { id: number; progress: number }) => void,
): Promise<UnlistenFn> {
  return listen<{ id: number; progress: number }>("download_update", (evt) =>
    cb(evt.payload),
  );
}
