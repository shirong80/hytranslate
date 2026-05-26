import { create } from 'zustand';

import type { AppError } from '@lib/ipc/errors';

import {
  deleteAllTranslationRecords,
  deleteTranslationRecord,
  exportHistoryCsv,
  exportHistoryJson,
  listTranslationRecords,
  searchTranslationRecords,
  setTags as setTagsIpc,
  toggleFavorite as toggleFavoriteIpc,
} from './ipc';
import { HISTORY_PAGE_SIZE, type ExportResult, type TranslationRecord } from './types';

interface Filters {
  favoriteOnly: boolean;
  tag: string | null;
}

export interface HistoryState {
  records: TranslationRecord[];
  total: number;
  loading: boolean;
  error: AppError | null;
  /** 마지막으로 적용된 검색어. 빈 문자열이면 mode === 'list'. */
  query: string;
  filters: Filters;
  selectedId: string | null;
  lastExport: ExportResult | null;
}

export interface HistoryActions {
  fetch: () => Promise<void>;
  setQuery: (q: string) => void;
  setFavoriteFilter: (on: boolean) => void;
  setTagFilter: (tag: string | null) => void;
  selectRecord: (id: string | null) => void;
  toggleFavorite: (id: string) => Promise<void>;
  setTags: (id: string, tags: string[]) => Promise<void>;
  removeRecord: (id: string) => Promise<void>;
  removeAll: () => Promise<void>;
  exportCsv: () => Promise<void>;
  exportJson: () => Promise<void>;
  /** 외부에서 record 삽입을 통보받았을 때 (translation:completed) — 첫 페이지에 prepend. */
  prependRecord: (record: TranslationRecord) => void;
  reset: () => void;
}

const initialState: HistoryState = {
  records: [],
  total: 0,
  loading: false,
  error: null,
  query: '',
  filters: { favoriteOnly: false, tag: null },
  selectedId: null,
  lastExport: null,
};

export const useHistoryStore = create<HistoryState & HistoryActions>()((set, get) => ({
  ...initialState,

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const { query, filters } = get();
      const tag = filters.tag?.trim() ? filters.tag.trim() : null;
      const request = {
        limit: HISTORY_PAGE_SIZE,
        offset: 0,
        favoriteOnly: filters.favoriteOnly,
        tag,
      };
      const result = query.trim()
        ? await searchTranslationRecords({ ...request, query: query.trim() })
        : await listTranslationRecords(request);

      const stillExists = result.records.some((r) => r.id === get().selectedId);
      set({
        records: result.records,
        total: result.total,
        loading: false,
        selectedId: stillExists ? get().selectedId : (result.records[0]?.id ?? null),
      });
    } catch (err) {
      set({ error: toAppError(err), loading: false });
    }
  },

  setQuery: (q) => {
    set({ query: q });
  },

  setFavoriteFilter: (on) => {
    set({ filters: { ...get().filters, favoriteOnly: on } });
  },

  setTagFilter: (tag) => {
    set({ filters: { ...get().filters, tag } });
  },

  selectRecord: (id) => {
    set({ selectedId: id });
  },

  toggleFavorite: async (id) => {
    try {
      const next = await toggleFavoriteIpc(id);
      if (next === null) return;
      set({
        records: get().records.map((r) => (r.id === id ? { ...r, isFavorite: next } : r)),
      });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  setTags: async (id, tags) => {
    try {
      const ok = await setTagsIpc(id, tags);
      if (!ok) return;
      const normalized = normalize(tags);
      set({
        records: get().records.map((r) => (r.id === id ? { ...r, tags: normalized } : r)),
      });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  removeRecord: async (id) => {
    try {
      const ok = await deleteTranslationRecord(id);
      if (!ok) return;
      const records = get().records.filter((r) => r.id !== id);
      set({
        records,
        total: Math.max(get().total - 1, 0),
        selectedId: get().selectedId === id ? (records[0]?.id ?? null) : get().selectedId,
      });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  removeAll: async () => {
    try {
      const n = await deleteAllTranslationRecords();
      set({ records: [], total: Math.max(get().total - n, 0), selectedId: null });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  exportCsv: async () => {
    try {
      const res = await exportHistoryCsv();
      set({ lastExport: res, error: null });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  exportJson: async () => {
    try {
      const res = await exportHistoryJson();
      set({ lastExport: res, error: null });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  prependRecord: (record) => {
    const dedup = get().records.filter((r) => r.id !== record.id);
    set({ records: [record, ...dedup], total: get().total + 1 });
  },

  reset: () => {
    set(initialState);
  },
}));

function normalize(tags: string[]): string[] {
  const out: string[] = [];
  for (const raw of tags) {
    const t = raw.trim();
    if (!t) continue;
    if (!out.includes(t)) out.push(t);
  }
  return out;
}

function toAppError(err: unknown): AppError {
  if (typeof err === 'object' && err !== null && 'kind' in err) {
    return err as AppError;
  }
  return { kind: 'Internal', message: err instanceof Error ? err.message : String(err) };
}
