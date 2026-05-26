import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useTranslationStore } from '@features/translation/store';

import { useAutoCopyTranslation } from './use-auto-copy-translation';

const writeTextMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: (...args: unknown[]) => writeTextMock(...args),
}));

describe('useAutoCopyTranslation', () => {
  beforeEach(() => {
    writeTextMock.mockClear();
    useTranslationStore.getState().reset();
  });
  afterEach(() => {
    useTranslationStore.getState().reset();
  });

  it('does not copy when disabled', () => {
    renderHook(() => useAutoCopyTranslation(false));
    act(() => {
      useTranslationStore.getState().beginRequest('r-1');
      useTranslationStore.getState().markCompleted({
        requestId: 'r-1',
        fullText: 'hello',
        durationMs: 10,
      });
    });
    expect(writeTextMock).not.toHaveBeenCalled();
  });

  it('copies once when enabled and translation completes', async () => {
    renderHook(() => useAutoCopyTranslation(true));
    await act(async () => {
      useTranslationStore.getState().beginRequest('r-2');
      useTranslationStore.getState().markCompleted({
        requestId: 'r-2',
        fullText: 'hello world',
        durationMs: 10,
      });
    });
    expect(writeTextMock).toHaveBeenCalledTimes(1);
    expect(writeTextMock).toHaveBeenCalledWith('hello world');
  });

  it('does not copy on cancellation or error', async () => {
    renderHook(() => useAutoCopyTranslation(true));
    await act(async () => {
      useTranslationStore.getState().beginRequest('r-3');
      useTranslationStore.getState().markCancelled({ requestId: 'r-3' });
    });
    expect(writeTextMock).not.toHaveBeenCalled();

    await act(async () => {
      useTranslationStore.getState().beginRequest('r-4');
      useTranslationStore.getState().markError({
        requestId: 'r-4',
        error: { kind: 'OllamaNotRunning' },
      });
    });
    expect(writeTextMock).not.toHaveBeenCalled();
  });

  it('does not re-copy the same completed translation', async () => {
    const { rerender } = renderHook(({ enabled }) => useAutoCopyTranslation(enabled), {
      initialProps: { enabled: true },
    });
    await act(async () => {
      useTranslationStore.getState().beginRequest('r-5');
      useTranslationStore.getState().markCompleted({
        requestId: 'r-5',
        fullText: 'one shot',
        durationMs: 10,
      });
    });
    expect(writeTextMock).toHaveBeenCalledTimes(1);
    // 재렌더만 일어나도 같은 결과 한번만 복사.
    rerender({ enabled: true });
    expect(writeTextMock).toHaveBeenCalledTimes(1);
  });
});
