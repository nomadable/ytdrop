import type { Download } from "../types";
import { DownloadItem } from "./DownloadItem";

export function QueueList({
  items,
  onRemove,
}: {
  items: Download[];
  onRemove: (id: number) => void;
}) {
  if (items.length === 0) return null;
  const active = items.filter((i) => i.status === "downloading");
  const queued = items.filter((i) => i.status === "queued");
  return (
    <section className="section">
      {active.length > 0 && (
        <>
          <h2>진행 중 ({active.length})</h2>
          {active.map((i) => (
            <DownloadItem key={i.id} item={i} />
          ))}
        </>
      )}
      {queued.length > 0 && (
        <>
          <h2>대기 중 ({queued.length})</h2>
          {queued.map((i) => (
            <DownloadItem
              key={i.id}
              item={i}
              actions={<button onClick={() => onRemove(i.id)}>✕</button>}
            />
          ))}
        </>
      )}
    </section>
  );
}
