import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn();
const listenMock = vi.fn().mockResolvedValue(() => undefined);

vi.mock('@lib/ipc/client', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  listen: (...args: unknown[]) => listenMock(...args),
}));

import { useTranslationStore } from './store';
import { useTranslationController } from './use-translation-controller';

describe('useTranslationController', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    listenMock.mockClear();
    useTranslationStore.getState().reset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('debounces typing and only invokes translate_stream after the last input', async () => {
    renderHook(() => useTranslationController({ debounceMs: 500 }));

    // 빠른 연속 입력 — Critical 2 / §15 race 시나리오.
    act(() => {
      useTranslationStore.getState().setSourceText('안');
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    act(() => {
      useTranslationStore.getState().setSourceText('안녕');
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    act(() => {
      useTranslationStore.getState().setSourceText('안녕하세요');
    });
    // 500ms 가 다 지나기 전에는 호출 없음.
    expect(invokeMock).not.toHaveBeenCalledWith('translate_stream', expect.anything());

    await act(async () => {
      await vi.advanceTimersByTimeAsync(600);
    });

    const translateCalls = invokeMock.mock.calls.filter((c) => c[0] === 'translate_stream');
    expect(translateCalls).toHaveLength(1);
    // 마지막 입력이 사용돼야 한다.
    expect((translateCalls[0]?.[1] as { request: { sourceText: string } }).request.sourceText).toBe(
      '안녕하세요',
    );
  });

  it('sends cancel_translation on input change while a request is in flight', async () => {
    // 첫 입력 → debounce 후 translate_stream. 그 다음 입력 변경 시 cancel.
    let inFlightRequestId = '';
    invokeMock.mockImplementation(async (cmd: string, payload: unknown) => {
      if (cmd === 'translate_stream') {
        inFlightRequestId = (payload as { request: { requestId: string } }).request.requestId;
      }
      return undefined;
    });

    renderHook(() => useTranslationController({ debounceMs: 500 }));

    act(() => {
      useTranslationStore.getState().setSourceText('hello');
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(600);
    });
    expect(inFlightRequestId).not.toBe('');

    // 입력 변경 → 즉시 cancel.
    act(() => {
      useTranslationStore.getState().setSourceText('helloo');
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    const cancelCalls = invokeMock.mock.calls.filter((c) => c[0] === 'cancel_translation');
    const cancelledIds = cancelCalls.map((c) => (c[1] as { requestId: string }).requestId);
    expect(cancelledIds).toContain(inFlightRequestId);
  });
});
