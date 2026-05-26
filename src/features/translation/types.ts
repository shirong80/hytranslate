// dropdown 순서: Auto 가 첫 항목 — Phase 2 기본값.
export const SOURCE_LANGUAGES = [
  'Auto',
  'Korean',
  'ChineseSimplified',
  'ChineseTraditional',
] as const;
export type SourceLanguage = (typeof SOURCE_LANGUAGES)[number];

export type TranslationStatus =
  | 'idle'
  | 'typing'
  | 'detecting'
  | 'translating'
  | 'completed'
  | 'cancelled'
  | 'error';

export const MAIN_INPUT_LIMIT = 30_000;
export const POPUP_INPUT_LIMIT = 5_000;
export const MENUBAR_INPUT_LIMIT = 5_000;

export const DEFAULT_MODEL = 'hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M';

export interface TranslateRequest {
  sourceText: string;
  sourceLanguage: SourceLanguage;
  model: string;
  requestId: string;
}

export interface StartedPayload {
  requestId: string;
  model: string;
  startedAtMs: number;
  resolvedLanguage: SourceLanguage;
}

export interface ChunkPayload {
  requestId: string;
  delta: string;
}

export interface CompletedPayload {
  requestId: string;
  fullText: string;
  durationMs: number;
}

export interface CancelledPayload {
  requestId: string;
}

export interface ErrorPayload {
  requestId: string;
  error: import('@lib/ipc/errors').AppError;
}
