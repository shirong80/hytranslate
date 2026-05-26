-- HyTranslate Mac Phase 4 — 초기 이력 스키마 (PRD §9.1 / §9.4).
--
-- 설계 메모:
-- - `rowid` 를 명시적으로 INTEGER PRIMARY KEY 로 alias 해서 FTS5 content_rowid 매핑에 쓴다.
--   PRD 의 `id` (UUID v4) 는 FE/Repo 가 노출하는 외부 식별자.
-- - FTS5 external-content 모드 — 인덱스만 별도 저장하고 본문은 translation_records 가 소유.
-- - 트리거 3개로 insert/update/delete 시 FTS 인덱스를 동기화한다. 외부에서 raw SQL 로
--   translation_records 를 만지지 않는 한 drift 가 발생하지 않는다.

CREATE TABLE translation_records (
    rowid           INTEGER PRIMARY KEY AUTOINCREMENT,
    id              TEXT    NOT NULL UNIQUE,
    source_text     TEXT    NOT NULL,
    source_language TEXT    NOT NULL,
    translated_text TEXT    NOT NULL,
    model           TEXT    NOT NULL,
    duration_ms     INTEGER NOT NULL,
    created_at      TEXT    NOT NULL,
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    tags_json       TEXT    NOT NULL DEFAULT '[]'
);

CREATE INDEX idx_translation_records_created_at ON translation_records (created_at DESC);
CREATE INDEX idx_translation_records_favorite   ON translation_records (is_favorite, created_at DESC);

CREATE VIRTUAL TABLE translation_records_fts USING fts5(
    source_text,
    translated_text,
    content='translation_records',
    content_rowid='rowid',
    tokenize='unicode61'
);

CREATE TRIGGER translation_records_ai AFTER INSERT ON translation_records BEGIN
    INSERT INTO translation_records_fts (rowid, source_text, translated_text)
    VALUES (new.rowid, new.source_text, new.translated_text);
END;

CREATE TRIGGER translation_records_ad AFTER DELETE ON translation_records BEGIN
    INSERT INTO translation_records_fts (translation_records_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.rowid, old.source_text, old.translated_text);
END;

CREATE TRIGGER translation_records_au AFTER UPDATE ON translation_records BEGIN
    INSERT INTO translation_records_fts (translation_records_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.rowid, old.source_text, old.translated_text);
    INSERT INTO translation_records_fts (rowid, source_text, translated_text)
    VALUES (new.rowid, new.source_text, new.translated_text);
END;
