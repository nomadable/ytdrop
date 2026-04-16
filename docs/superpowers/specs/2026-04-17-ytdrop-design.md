# ytdrop — 크로스 플랫폼 yt-dlp 데스크탑 앱 설계

## 1. 개요

macOS와 Windows에서 동작하는 단일 창 데스크탑 앱. 사용자가 YouTube(및 yt-dlp가 지원하는 기타 사이트) URL을 붙여넣으면 큐에 추가되어 순차적으로 mp4 파일로 다운로드한다. 기존 zsh `ytdl()` 함수의 동작을 그대로 재현하되, 터미널이 없는 환경에서도 사용할 수 있도록 GUI와 큐·히스토리·진행률 표시를 제공한다.

**참조 원본 스크립트**

```sh
yt-dlp -f "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best" \
  --merge-output-format mp4 \
  --no-playlist \
  --replace-in-metadata title " " "_" \
  -o "$HOME/Downloads/%(title)s.%(ext)s" \
  "$1"
```

## 2. 사용자와 성공 기준

- **1차 사용자**: 작성자 본인의 개인 유틸리티.
- **2차 사용자**: 지인에게 공유(비기술자 가정).
- **성공 기준**: 설치 직후 터미널·패키지 매니저 없이 바로 동작해야 한다. URL 붙여넣기 → 다운로드 완료까지 1개의 창 안에서 끝난다.

## 3. 범위

### 3.1 포함

- URL 붙여넣기 기반 다운로드 시작 (Enter/버튼)
- 다운로드 큐(동시 1개, 순차 처리)와 실시간 진행률
- 완료/실패 히스토리 (영구 보관)
- 저장 폴더 변경 (유일한 사용자 설정)
- 완료 시 OS 네이티브 알림
- 실패 시 재시도 버튼, 큐에서 제거 버튼
- 파일/폴더 바로 열기 (히스토리)

### 3.2 제외 (YAGNI)

- 포맷·품질 선택 UI — 원 스크립트 설정 고정
- 플레이리스트 다운로드 — `--no-playlist` 고정
- 동시 다운로드(병렬) — 큐 1개씩만
- 자동 업데이트 기능
- 코드 사이닝 (초기 버전)
- 클립보드 감지, 드래그&드롭, 전역 단축키 — 명시적 붙여넣기만
- 인증이 필요한 사이트의 쿠키 지원
- 자막·섬네일 별도 저장

## 4. UI 설계

### 4.1 화면 구성 (단일 창, 탭 없음)

```
┌──────────────────────────────────────────┐
│  [ URL 붙여넣기...        ] [ 다운로드 ] │  상단 입력줄
├──────────────────────────────────────────┤
│  ▶ 진행 중 (1)                           │
│    ● [썸네일] 영상 제목        63% ──▓▓░│
│                                          │
│  ⏳ 대기 중 (2)                          │
│    ○ [썸네일] 다른 영상 제목     [x]    │  x: 큐에서 제거
│    ○ [썸네일] 또 다른 영상       [x]    │
├──────────────────────────────────────────┤
│  ✅ 최근 다운로드 (접기/펼치기)          │
│    · 영상 제목  [파일 열기][폴더 열기]   │
│    · ... (실패 항목은 [재시도] 버튼)     │
└──────────────────────────────────────────┘
  ⚙️ 설정 아이콘 → 저장 폴더 변경 다이얼로그
```

### 4.2 상호작용 흐름

1. 사용자가 URL 입력 필드에 붙여넣기 → Enter 또는 `다운로드` 클릭.
2. 앱이 `yt-dlp --dump-json`으로 메타데이터(제목, 썸네일) 사전 조회.
   - 실패: 입력 필드 아래 빨간 토스트 "유효한 URL이 아닙니다". 큐에 추가하지 않음.
   - 성공: DB에 `status=queued`로 row 생성, 제목·썸네일 저장, UI에 즉시 반영.
3. 큐 워커가 `status=queued` 항목을 오래된 순으로 1개씩 처리한다.
4. 진행 중 항목은 `status=downloading`, `progress`가 0.0~1.0으로 실시간 업데이트.
5. 완료 시 `status=completed`, `file_path` 저장, OS 네이티브 알림 발송. UI에서 항목이 "최근 다운로드"로 이동.
6. 실패 시 `status=failed`, `error`에 stderr 마지막 줄 저장. "최근 다운로드"에 빨간 상태로 노출되며 `재시도` 버튼으로 `status=queued` 복귀.

### 4.3 고정 다운로드 옵션 (UI 비노출)

원 `ytdl()` 함수 그대로:

- `-f "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best"`
- `--merge-output-format mp4`
- `--no-playlist`
- `--replace-in-metadata title " " "_"`
- 파일명 템플릿: `%(title)s.%(ext)s`
- 저장 경로 루트는 설정값(기본 `~/Downloads`)

## 5. 기술 구조

### 5.1 스택

- **Tauri 2.x**
- **Frontend**: React + TypeScript (상태 관리는 내장 state로 충분, 외부 상태 라이브러리 없음)
- **Backend**: Rust (sidecar 실행, SQLite I/O, 이벤트 emit)

### 5.2 구성 요소

```
┌─ Frontend (React/TS) ──────────────────────┐
│ · 큐/히스토리 UI                            │
│ · invoke("start_download", {url})           │
│ · invoke("retry_download", {id})            │
│ · invoke("remove_from_queue", {id})         │
│ · invoke("open_file"/"open_folder", {id})   │
│ · invoke("get_settings"/"set_download_dir") │
│ · listen("download_update", e => ...)       │
└────────────────┬───────────────────────────┘
                 │ Tauri IPC
┌────────────────▼───────────────────────────┐
│ Rust Backend (src-tauri/)                  │
│ · commands: start/retry/remove/open/settings│
│ · queue worker: 단일 tokio 태스크           │
│ · yt-dlp 실행, stdout 라인 파싱 → 이벤트    │
│ · SQLite 접근 (rusqlite)                    │
└────────────────┬───────────────────────────┘
                 │ spawn
┌────────────────▼───────────────────────────┐
│ Sidecar 바이너리 (src-tauri/binaries/)      │
│ · yt-dlp-{target triple}                   │
│ · ffmpeg-{target triple}                   │
└────────────────────────────────────────────┘
```

### 5.3 데이터 모델 (SQLite 단일 테이블)

```sql
CREATE TABLE downloads (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  url          TEXT    NOT NULL,
  title        TEXT,
  thumbnail    TEXT,           -- 원격 URL (초기엔 캐시 없음)
  file_path    TEXT,           -- 완료 시 저장된 절대 경로
  status       TEXT    NOT NULL CHECK (status IN ('queued','downloading','completed','failed')),
  progress     REAL    NOT NULL DEFAULT 0,  -- 0.0 ~ 1.0
  error        TEXT,
  created_at   INTEGER NOT NULL,           -- unix millis
  completed_at INTEGER
);
CREATE INDEX idx_status_created ON downloads(status, created_at);
```

진행 중/대기 중/히스토리는 모두 `status`의 필터일 뿐. UI 각 섹션은 해당 상태로 쿼리.

### 5.4 설정 영속화

Tauri의 앱 설정 디렉토리에 JSON 파일 하나:

- macOS: `~/Library/Application Support/com.nomadable.ytdrop/settings.json`
- Windows: `%APPDATA%\com.nomadable.ytdrop\settings.json`

```json
{ "downloadDir": "/Users/k.jeong/Downloads" }
```

존재하지 않으면 OS의 Downloads 폴더를 기본값으로 채워 생성.

### 5.5 yt-dlp 실행 인자

```
<yt-dlp sidecar> <URL>
  -f "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best"
  --merge-output-format mp4
  --no-playlist
  --replace-in-metadata title " " "_"
  --ffmpeg-location <ffmpeg sidecar 절대 경로>
  --newline
  --progress-template "download:%(progress._percent_str)s"
  -o "<downloadDir>/%(title)s.%(ext)s"
```

- `--newline`: 진행률이 캐리지 리턴이 아닌 개행으로 출력되어 라인 단위 파싱이 쉬워짐.
- `--progress-template`: 간결한 형식으로 출력. 예: `download: 63.2%` → 정규식으로 숫자만 추출, 0.632로 정규화.
- 메타데이터 조회용 호출은 같은 sidecar로 `--dump-json --no-playlist --skip-download <URL>`.

### 5.6 큐 워커

- 앱 시작 시 Rust 쪽에서 단일 tokio 태스크를 spawn.
- `status=downloading` 상태로 남아있는 항목은 앱 비정상 종료의 잔재 → 시작 시 `queued`로 복구.
- 루프:
  1. `status='queued' ORDER BY created_at ASC LIMIT 1` 조회.
  2. 없으면 notify 채널에서 새 항목 신호 대기.
  3. 있으면 `status='downloading'`으로 업데이트, yt-dlp 실행, 진행률 이벤트 emit, 종료 코드에 따라 `completed` 또는 `failed`로 업데이트.

## 6. 에러 처리

| 상황 | 처리 |
|------|------|
| URL 유효성 실패 (`--dump-json` 실패) | 큐 미추가. 입력 필드 하단에 에러 메시지 토스트. |
| 다운로드 실행 실패 (네트워크/서버) | `status=failed`, stderr 마지막 줄을 `error`에 저장. 재시도 버튼 노출. |
| ffmpeg 병합 실패 (exit != 0) | 다운로드 실행 실패와 동일 경로. |
| 디스크 공간 부족 | 동일 경로, 에러 문자열 그대로 노출. 별도 휴리스틱 분기 없음. |
| sidecar 바이너리 없음 | 앱 시작 시 경로 검증. 없으면 치명 다이얼로그: "앱을 재설치해주세요". |
| 앱 비정상 종료 중 다운로드 중단 | 시작 시 `downloading` → `queued`로 복구. 파일은 yt-dlp의 `.part`로 남음; 재개는 yt-dlp 기본 동작에 위임. |

## 7. 의존성 번들링

### 7.1 번들 대상 바이너리

- `yt-dlp` 공식 릴리스의 standalone 바이너리
  - `yt-dlp_macos_legacy` (Intel) / `yt-dlp_macos` (Apple Silicon) / `yt-dlp.exe` (Windows)
- `ffmpeg` 정적 빌드 (예: `evermeet.cx/ffmpeg` for macOS, `gyan.dev` 또는 BtbN 빌드 for Windows)

### 7.2 배치

`src-tauri/binaries/` 하위에 Tauri 컨벤션에 맞춰 `{name}-{target-triple}` 형식으로 배치:

```
src-tauri/binaries/
  yt-dlp-x86_64-apple-darwin
  yt-dlp-aarch64-apple-darwin
  yt-dlp-x86_64-pc-windows-msvc.exe
  ffmpeg-x86_64-apple-darwin
  ffmpeg-aarch64-apple-darwin
  ffmpeg-x86_64-pc-windows-msvc.exe
```

`tauri.conf.json`의 `bundle.externalBin`에 등록. 런타임에 `Command::new_sidecar("yt-dlp")` 등으로 접근.

### 7.3 예상 앱 크기

- 프레임워크: 10~15MB
- yt-dlp: ~25MB
- ffmpeg (정적): ~60~80MB
- **최종 인스톨러 크기 예상**: macOS DMG 70~90MB, Windows MSI 80~100MB

## 8. 배포

- **빌드**: `tauri build`. 초기엔 로컬 수동 빌드.
- **산출물**:
  - macOS: 아키텍처별 `.dmg` 2종 (Apple Silicon, Intel). 초기엔 ad-hoc 서명.
  - Windows: `.msi` 인스톨러. 초기엔 무서명.
- **설치 안내 (README)**:
  - macOS: "앱 우클릭 → 열기 → 열기 확인" (Gatekeeper 최초 우회).
  - Windows: SmartScreen "추가 정보 → 실행".
- **배포 채널**: GitHub Releases. 지인에게는 릴리스 URL 공유.

## 9. 테스트 전략 (최소)

개인 유틸 범위라 무거운 테스트 스위트는 만들지 않는다. 최소한:

- **단위 테스트 (Rust)**: 진행률 파싱 함수, 설정 파일 로드/저장, SQLite 스키마 마이그레이션.
- **수동 QA 체크리스트**: 유효 URL/무효 URL/플레이리스트 URL/네트워크 끊김/앱 재시작 중 다운로드 재개 등. 릴리스 전 체크.
- **프론트엔드**: React 컴포넌트 단위 테스트는 생략. UI는 수동 확인.

## 10. 미해결 / 추후 결정 사항

- **썸네일 캐싱**: 초기엔 원격 URL을 그대로 `<img>`에 노출. 오프라인에서 히스토리가 깨지면 그때 로컬 캐시 도입.
- **yt-dlp 업그레이드**: 초기엔 앱 릴리스 주기에 맞춰 번들 재빌드. 자가 업데이트는 초기 범위 외.
- **macOS 유니버설 바이너리**: 초기엔 arch별 2종. 나중에 유니버설로 통합 고려.
- **로그 수집**: 초기엔 Tauri의 표준 로그(stdout)에만 기록. 별도 로그 파일/원격 전송 없음.
