import { useCallback, useEffect, useState } from "react";
import { api, onDownloadsChanged, onProgress } from "./api";
import type { Download, Settings } from "./types";
import { UrlInput } from "./components/UrlInput";
import { QueueList } from "./components/QueueList";
import { HistoryList } from "./components/HistoryList";
import { SettingsDialog } from "./components/SettingsDialog";

export default function App() {
  const [items, setItems] = useState<Download[]>([]);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const refresh = useCallback(async () => {
    setItems(await api.list());
  }, []);

  useEffect(() => {
    refresh();
    api.getSettings().then(setSettings);
    const unlisten1 = onDownloadsChanged(refresh);
    const unlisten2 = onProgress(({ id, progress }) => {
      setItems((prev) =>
        prev.map((it) => (it.id === id ? { ...it, progress } : it)),
      );
    });
    return () => {
      unlisten1.then((u) => u());
      unlisten2.then((u) => u());
    };
  }, [refresh]);

  const queued = items.filter(
    (i) => i.status === "queued" || i.status === "downloading",
  );
  const history = items.filter(
    (i) => i.status === "completed" || i.status === "failed",
  );

  return (
    <div className="app">
      <header className="topbar">
        <h1>ytdrop</h1>
        <button
          className="icon"
          onClick={() => setSettingsOpen(true)}
          title="설정"
        >
          ⚙️
        </button>
      </header>
      <UrlInput
        onSubmit={async (url) => {
          await api.startDownload(url);
        }}
      />
      <QueueList items={queued} onRemove={(id) => api.remove(id)} />
      <HistoryList
        items={history}
        onRetry={(id) => api.retry(id)}
        onClear={() => api.clearHistory()}
      />
      {settingsOpen && settings && (
        <SettingsDialog
          settings={settings}
          onClose={() => setSettingsOpen(false)}
          onChanged={(s) => setSettings(s)}
        />
      )}
    </div>
  );
}
