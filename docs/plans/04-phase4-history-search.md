# Phase 4 — 이력과 검색

PRD §15.4 / §6.6 / §7.6 / §8.7 / §9.1 / §9.4 / §10.6 구현.

## 목표

- 완료된 번역 결과를 SQLite 에 저장하고 다시 찾아 쓸 수 있게 한다.
- v1 잠금 결정 (`docs/CLAUDE.md` 표 + PRD §18):
  - 이력 저장 기본 ON
  - DB 암호화 OUT (v1 제외)
  - DB 위치: `~/Library/Application Support/<bundle id>/hytranslate.sqlite`
  - FTS5 사용

## 잠금 결정 (Phase 4 한정)

| 항목 | 결정 |
|---|---|
| DB 풀 | `r2d2` + `r2d2_sqlite` 단일 풀. setup() 에서 초기화. |
| 마이그레이션 | numbered SQL `include_str!` 임베드. `PRAGMA user_version` 으로 추적. |
| FTS5 | `content='translation_records'` external-content, rowid 매핑 + 동기화 트리거. |
| Tags | JSON 문자열 1 컬럼 (`tags_json`). 필터는 `LIKE '%"tag"%'` (v1 충분). |
| 정렬 | `created_at DESC` 기본. 즐겨찾기/태그/검색 모두 동일. |
| Export | `tauri-plugin-dialog` 로 사용자 경로 선택. CSV / JSON 두 종류. |
| 페이징 | `limit` + `offset` (FE 가 무한스크롤 또는 페이지 버튼). v1 은 단순 `limit=50` 기본. |
| 저장 시점 | `translation:completed` 이후, `settings.save_history` 가 켜진 경우에만 insert. cancel/error 는 저장하지 않음. |
| 화면 | 메인 윈도우에 `history` route 추가 (translate / history / settings 3 탭). 별도 윈도우 새로 만들지 않음. |

## 데이터 모델 (PRD §9.1)

```sql
CREATE TABLE translation_records (
    id              TEXT PRIMARY KEY,
    source_text     TEXT NOT NULL,
    source_language TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    model           TEXT NOT NULL,
    duration_ms     INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    tags_json       TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX idx_translation_records_created_at ON translation_records (created_at DESC);
CREATE INDEX idx_translation_records_favorite  ON translation_records (is_favorite, created_at DESC);

CREATE VIRTUAL TABLE translation_records_fts USING fts5(
    source_text, translated_text,
    content='translation_records', content_rowid='rowid'
);

-- INSERT / UPDATE / DELETE 트리거로 FTS 동기화.
```

## Backend 구조

- `src-tauri/src/db/`
  - `mod.rs` — 풀 초기화, 마이그레이션 러너
  - `migrations/0001_init.sql`
- `src-tauri/src/history/`
  - `mod.rs` — `HistoryRepo`, `TranslationRecord`, CRUD + search + export-iterator
- `src-tauri/src/commands/history.rs` — Tauri 어댑터
- `src-tauri/src/commands/translate.rs` — completed 이후 repo.insert 호출 (gated)

## Frontend 구조

- `src/features/history/`
  - `types.ts` — `TranslationRecord`, pagination types
  - `ipc.ts` — 9 개 invoke 래퍼
  - `store.ts` — list state, query, filters, selection
  - `components/history-panel.tsx`
  - `components/history-list.tsx`
  - `components/history-detail.tsx`
- `src/windows/main/main.tsx` — `Route` 에 `'history'` 추가, nav 버튼 추가

## 완료 기준 (PRD §15.4)

- [x] 완료된 번역이 SQLite 에 저장된다.
- [x] 검색, favorite, tag 필터가 동작한다.
- [x] 전체 삭제가 동작한다.
- [x] 이력 저장 OFF 에서는 새 기록이 저장되지 않는다.

## 테스트 (PRD §14.3)

- 마이그레이션 forward apply 테스트
- FTS5 insert + 검색 ranking 테스트
- delete / delete_all 테스트
- favorite 토글 + tags round-trip 테스트
- save_history OFF 시 insert skip 테스트
- export CSV / JSON 직렬화 테스트
- FE store: list, search, filter, mutation 테스트
- FE ipc.ts mocked invoke 테스트
