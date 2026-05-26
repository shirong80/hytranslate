import { beforeEach, describe, expect, it, vi } from 'vitest';

import { listTranslationRecords } from './ipc';

const invokeMock = vi.fn();
vi.mock('@lib/ipc/client', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

describe('menubar recent (list_translation_records) — code-review v1 follow-up §6 + review §25', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('menubar 5건 조회를 list_translation_records 로 요청', async () => {
    invokeMock.mockResolvedValue({ records: [], total: 0 });
    await listTranslationRecords({ limit: 5 });
    expect(invokeMock).toHaveBeenCalledWith('list_translation_records', {
      request: { limit: 5 },
    });
  });

  it('빈 응답이면 빈 리스트', async () => {
    invokeMock.mockResolvedValue({ records: [], total: 0 });
    const result = await listTranslationRecords({ limit: 5 });
    expect(result.records).toHaveLength(0);
    expect(result.total).toBe(0);
  });

  it('TranslationRecord 의 sourceText / translatedText 가 menubar 가 기대하는 키로 도착', async () => {
    invokeMock.mockResolvedValue({
      records: [
        {
          id: 'r1',
          sourceText: '안녕',
          sourceLanguage: 'Korean',
          translatedText: 'Hello',
          model: 'm',
          durationMs: 100,
          createdAt: '2026-05-26T00:00:00Z',
          isFavorite: false,
          tags: [],
        },
      ],
      total: 1,
    });
    const result = await listTranslationRecords({ limit: 5 });
    expect(result.records[0]?.sourceText).toBe('안녕');
    expect(result.records[0]?.translatedText).toBe('Hello');
  });
});
