import type { Download } from "../types";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";

export function DownloadItem({
  item,
  actions,
}: {
  item: Download;
  actions?: React.ReactNode;
}) {
  const pct = Math.round((item.progress ?? 0) * 100);
  return (
    <div className={`item status-${item.status}`}>
      {item.thumbnail ? (
        <img className="thumb" src={item.thumbnail} alt="" />
      ) : (
        <div className="thumb placeholder" />
      )}
      <div className="meta">
        <div className="title" title={item.url}>
          {item.title ?? item.url}
        </div>
        {item.status === "downloading" && (
          <div className="progress">
            <div className="bar" style={{ width: `${pct}%` }} />
            <span>{pct}%</span>
          </div>
        )}
        {item.status === "failed" && (
          <div className="err">{item.error ?? "실패"}</div>
        )}
        {item.status === "completed" && item.filePath && (
          <div className="actions">
            <button onClick={() => openPath(item.filePath!)}>파일 열기</button>
            <button onClick={() => revealItemInDir(item.filePath!)}>
              폴더 열기
            </button>
          </div>
        )}
      </div>
      <div className="tail">{actions}</div>
    </div>
  );
}
