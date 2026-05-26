import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useHistoryStore } from './store';
import type { ListResult, TranslationRecord } from './types';

const mocks = vi.hoisted(() => ({
  list: vi.fn(),
  search: vi.fn(),
  remove: vi.fn(),
  removeAll: vi.fn(),
  toggle: vi.fn(),
  setTags: vi.fn(),
  exportCsv: vi.fn(),
  exportJson: vi.fn(),
}));

vi.mock('./ipc', () => ({
  listTranslationRecords: mocks.list,
  searchTranslationRecords: mocks.search,
  deleteTranslationRecord: mocks.remove,
  deleteAllTranslationRecords: mocks.removeAll,
  toggleFavorite: mocks.toggle,
  setTags: mocks.setTags,
  exportHistoryCsv: mocks.exportCsv,
  exportHistoryJson: mocks.exportJson,
  getTranslationRecord: vi.fn(),
}));

function record(id: string, overrides: Partial<TranslationRecord> = {}): TranslationRecord {
  return {
    id,
    sourceText: `src-${id}`,
    sourceLanguage: 'Korean',
    translatedText: `dst-${id}`,
    model: 'm',
    durationMs: 42,
    createdAt: '2026-05-26T00:00:00Z',
    isFavorite: false,
    tags: [],
    ...overrides,
  };
}

function setListResult(records: TranslationRecord[]) {
  const result: ListResult = { records, total: records.length };
  mocks.list.mockResolvedValue(result);
  mocks.search.mockResolvedValue(result);
}

beforeEach(() => {
  useHistoryStore.getState().reset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe('useHistoryStore', () => {
  it('fetches via list when query is empty', async () => {
    setListResult([record('a'), record('b')]);
    await useHistoryStore.getState().fetch();
    expect(mocks.list).toHaveBeenCalledTimes(1);
    expect(mocks.search).not.toHaveBeenCalled();
    const state = useHistoryStore.getState();
    expect(state.records).toHaveLength(2);
    expect(state.total).toBe(2);
    expect(state.selectedId).toBe('a');
  });

  it('fetches via search when query is non-empty', async () => {
    setListResult([record('q1')]);
    useHistoryStore.getState().setQuery('thanks');
    await useHistoryStore.getState().fetch();
    expect(mocks.search).toHaveBeenCalledTimes(1);
    expect(mocks.list).not.toHaveBeenCalled();
  });

  it('toggleFavorite updates record in place when ipc returns boolean', async () => {
    setListResult([record('a'), record('b')]);
    await useHistoryStore.getState().fetch();
    mocks.toggle.mockResolvedValue(true);
    await useHistoryStore.getState().toggleFavorite('a');
    const a = useHistoryStore.getState().records.find((r) => r.id === 'a');
    expect(a?.isFavorite).toBe(true);
  });

  it('toggleFavorite no-ops when ipc returns null (missing row)', async () => {
    setListResult([record('a', { isFavorite: false })]);
    await useHistoryStore.getState().fetch();
    mocks.toggle.mockResolvedValue(null);
    await useHistoryStore.getState().toggleFavorite('a');
    const a = useHistoryStore.getState().records.find((r) => r.id === 'a');
    expect(a?.isFavorite).toBe(false);
  });

  it('setTags normalizes tags before pushing to state', async () => {
    setListResult([record('a')]);
    await useHistoryStore.getState().fetch();
    mocks.setTags.mockResolvedValue(true);
    await useHistoryStore.getState().setTags('a', ['  법무 ', '법무', '', '연구']);
    const a = useHistoryStore.getState().records.find((r) => r.id === 'a');
    expect(a?.tags).toEqual(['법무', '연구']);
  });

  it('removeRecord drops the row and reselects the next', async () => {
    setListResult([record('a'), record('b'), record('c')]);
    await useHistoryStore.getState().fetch();
    useHistoryStore.getState().selectRecord('b');
    mocks.remove.mockResolvedValue(true);
    await useHistoryStore.getState().removeRecord('b');
    const state = useHistoryStore.getState();
    expect(state.records.map((r) => r.id)).toEqual(['a', 'c']);
    expect(state.total).toBe(2);
    // selectedId was on b which got removed → falls back to first remaining ('a').
    expect(state.selectedId).toBe('a');
  });

  it('removeAll clears records', async () => {
    setListResult([record('a'), record('b')]);
    await useHistoryStore.getState().fetch();
    mocks.removeAll.mockResolvedValue(2);
    await useHistoryStore.getState().removeAll();
    const state = useHistoryStore.getState();
    expect(state.records).toHaveLength(0);
    expect(state.selectedId).toBeNull();
  });

  it('prependRecord puts new entry at the top and bumps total', async () => {
    setListResult([record('a')]);
    await useHistoryStore.getState().fetch();
    useHistoryStore.getState().prependRecord(record('new'));
    const state = useHistoryStore.getState();
    expect(state.records[0]?.id).toBe('new');
    expect(state.total).toBe(2);
  });

  it('captures AppError shape from ipc rejections', async () => {
    mocks.list.mockRejectedValue({ kind: 'Internal', message: 'boom' });
    await useHistoryStore.getState().fetch();
    expect(useHistoryStore.getState().error?.kind).toBe('Internal');
  });
});
