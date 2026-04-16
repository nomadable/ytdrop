import { useState } from "react";

export function UrlInput({
  onSubmit,
}: {
  onSubmit: (url: string) => Promise<void>;
}) {
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    const url = value.trim();
    if (!url) return;
    setBusy(true);
    setError(null);
    try {
      await onSubmit(url);
      setValue("");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="urlbar">
      <input
        type="text"
        placeholder="URL 붙여넣기..."
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") submit();
        }}
        disabled={busy}
      />
      <button onClick={submit} disabled={busy || !value.trim()}>
        {busy ? "확인 중..." : "다운로드"}
      </button>
      {error && <div className="toast error">{error}</div>}
    </div>
  );
}
