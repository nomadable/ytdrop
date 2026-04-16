export type DownloadStatus = "queued" | "downloading" | "completed" | "failed";

export interface Download {
  id: number;
  url: string;
  title: string | null;
  thumbnail: string | null;
  filePath: string | null;
  status: DownloadStatus;
  progress: number;
  error: string | null;
  createdAt: number;
  completedAt: number | null;
}

export interface Settings {
  downloadDir: string;
}
