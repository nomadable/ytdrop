import { useState } from "react";
import type { Download } from "../types";
import { DownloadItem } from "./DownloadItem";

export function HistoryList({
  items,
  onRetry,
  onClear,
}: {
  items: Download[];
  onRetry: (id: number) => void;
  onClear: () => void;
}) {
  const [open, setOpen] = useState(true);
  if (items.length === 0) return null;
  return (
    <section className="section">
      <h2 onClick={() => setOpen((v) => !v)} className="collapsible">
        {open ? "▼" : "▶"} 최근 다운로드 ({items.length})
        <button
          className="link"
          onClick={(e) => {
            e.stopPropagation();
            onClear();
          }}
        >
          기록 비우기
        </button>
      </h2>
      {open &&
        items.map((i) => (
          <DownloadItem
            key={i.id}
            item={i}
            actions={
              i.status === "failed" ? (
                <button onClick={() => onRetry(i.id)}>재시도</button>
              ) : null
            }
          />
        ))}
    </section>
  );
}
