import { invoke } from '@lib/ipc/client';

import type {
  ExportResult,
  ListRequest,
  ListResult,
  SearchRequest,
  TranslationRecord,
} from './types';

export async function listTranslationRecords(request: ListRequest = {}): Promise<ListResult> {
  return invoke<ListResult>('list_translation_records', { request });
}

export async function searchTranslationRecords(request: SearchRequest): Promise<ListResult> {
  return invoke<ListResult>('search_translation_records', { request });
}

export async function getTranslationRecord(id: string): Promise<TranslationRecord | null> {
  return invoke<TranslationRecord | null>('get_translation_record', { request: { id } });
}

export async function deleteTranslationRecord(id: string): Promise<boolean> {
  return invoke<boolean>('delete_translation_record', { request: { id } });
}

export async function deleteAllTranslationRecords(): Promise<number> {
  return invoke<number>('delete_all_translation_records');
}

export async function toggleFavorite(id: string): Promise<boolean | null> {
  return invoke<boolean | null>('toggle_favorite', { request: { id } });
}

export async function setTags(id: string, tags: string[]): Promise<boolean> {
  return invoke<boolean>('set_tags', { request: { id, tags } });
}

export async function exportHistoryCsv(): Promise<ExportResult> {
  return invoke<ExportResult>('export_history_csv');
}

export async function exportHistoryJson(): Promise<ExportResult> {
  return invoke<ExportResult>('export_history_json');
}
