import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import type { Settings } from "../types";

export function SettingsDialog({
  settings,
  onClose,
  onChanged,
}: {
  settings: Settings;
  onClose: () => void;
  onChanged: (s: Settings) => void;
}) {
  async function pick() {
    const chosen = await open({
      directory: true,
      multiple: false,
      defaultPath: settings.downloadDir,
    });
    if (typeof chosen === "string") {
      const updated = await api.setDownloadDir(chosen);
      onChanged(updated);
    }
  }
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>설정</h3>
        <label>저장 폴더</label>
        <div className="row">
          <code className="path">{settings.downloadDir}</code>
          <button onClick={pick}>변경...</button>
        </div>
        <div className="row end">
          <button onClick={onClose}>닫기</button>
        </div>
      </div>
    </div>
  );
}
