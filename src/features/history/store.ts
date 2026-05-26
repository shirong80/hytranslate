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

// 코드리뷰 Med 3 — overlapping fetch race 방지용 모듈 로컬 카운터.
// 매 fetch 시작 시 ++lastIssuedFetch 로 seq 를 잡고, 응답이 돌아오면
// seq < lastIssuedFetch 인 응답은 commit 하지 않는다 (newer 가 이미 commit 됐을 수 있으므로).
let lastIssuedFetch = 0;

export const useHistoryStore = create<HistoryState & HistoryActions>()((set, get) => ({
  ...initialState,

  fetch: async () => {
    const seq = ++lastIssuedFetch;
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

      if (seq !== lastIssuedFetch) {
        // 더 새로운 fetch 가 이미 시작됨 — stale 응답이므로 무시.
        return;
      }
      const stillExists = result.records.some((r) => r.id === get().selectedId);
      set({
        records: result.records,
        total: result.total,
        loading: false,
        selectedId: stillExists ? get().selectedId : (result.records[0]?.id ?? null),
      });
    } catch (err) {
      if (seq !== lastIssuedFetch) {
        // stale error 도 무시.
        return;
      }
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
      // 코드리뷰 Med 1 — favorite-only 필터가 켜진 상태에서 unfavorite 하면
      // row 가 현재 필터를 더 이상 만족하지 않으므로 즉시 records 에서 빼고
      // total / selectedId 를 재조정한다.
      const updated = get().records.map((r) => (r.id === id ? { ...r, isFavorite: next } : r));
      const { filters } = get();
      applyMutation(set, get, id, updated, (record) => matchesFilter(record, filters));
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  setTags: async (id, tags) => {
    try {
      const ok = await setTagsIpc(id, tags);
      if (!ok) return;
      const normalized = normalize(tags);
      // 코드리뷰 Med 1 — 활성 tag 필터가 있고 mutation 결과가 그 태그를 더 이상
      // 포함하지 않으면 row 를 records 에서 뺀다.
      const updated = get().records.map((r) => (r.id === id ? { ...r, tags: normalized } : r));
      const { filters } = get();
      applyMutation(set, get, id, updated, (record) => matchesFilter(record, filters));
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

function matchesFilter(record: TranslationRecord, filters: Filters): boolean {
  if (filters.favoriteOnly && !record.isFavorite) return false;
  const tag = filters.tag?.trim();
  if (tag && !record.tags.includes(tag)) return false;
  return true;
}

/**
 * mutation 결과를 commit 한다. 만약 활성 filter 가 mutated row 를 더 이상 포함하지
 * 않으면 records 에서 빼고 total / selectedId 도 재조정한다.
 */
function applyMutation(
  set: (partial: Partial<HistoryState>) => void,
  get: () => HistoryState & HistoryActions,
  id: string,
  updatedRecords: TranslationRecord[],
  predicate: (record: TranslationRecord) => boolean,
) {
  const target = updatedRecords.find((r) => r.id === id);
  if (target && !predicate(target)) {
    const filtered = updatedRecords.filter((r) => r.id !== id);
    set({
      records: filtered,
      total: Math.max(get().total - 1, 0),
      selectedId: get().selectedId === id ? (filtered[0]?.id ?? null) : get().selectedId,
    });
    return;
  }
  set({ records: updatedRecords });
}

function toAppError(err: unknown): AppError {
  if (typeof err === 'object' && err !== null && 'kind' in err) {
    return err as AppError;
  }
  return { kind: 'Internal', message: err instanceof Error ? err.message : String(err) };
}
