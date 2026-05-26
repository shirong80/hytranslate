import type { SourceLanguage } from '@features/translation/types';

export interface TranslationRecord {
  id: string;
  sourceText: string;
  sourceLanguage: SourceLanguage;
  translatedText: string;
  model: string;
  durationMs: number;
  createdAt: string;
  isFavorite: boolean;
  tags: string[];
}

export interface ListResult {
  records: TranslationRecord[];
  total: number;
}

export interface ListRequest {
  limit?: number;
  offset?: number;
  favoriteOnly?: boolean;
  tag?: string | null;
}

export interface SearchRequest extends ListRequest {
  query: string;
}

export interface ExportResult {
  /** dialog 취소 시 `null` — silent 처리 신호. */
  path: string | null;
  records: number;
}

export const HISTORY_PAGE_SIZE = 50;
