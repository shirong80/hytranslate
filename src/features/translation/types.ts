export const SOURCE_LANGUAGES = ['Korean', 'ChineseSimplified', 'ChineseTraditional'] as const;
export type SourceLanguage = (typeof SOURCE_LANGUAGES)[number];

export type TranslationStatus =
  | 'idle'
  | 'debouncing'
  | 'translating'
  | 'completed'
  | 'cancelled'
  | 'error';

export const MAIN_INPUT_LIMIT = 30_000;

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
