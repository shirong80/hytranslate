import { beforeEach, describe, expect, it } from 'vitest';

import { useTranslationStore } from './store';

const reset = () => useTranslationStore.getState().reset();

describe('translation store', () => {
  beforeEach(() => {
    reset();
  });

  it('starts idle with default Auto source language', () => {
    const state = useTranslationStore.getState();
    expect(state.status).toBe('idle');
    expect(state.sourceLanguage).toBe('Auto');
    expect(state.output).toBe('');
    expect(state.error).toBeNull();
  });

  it('transitions through start → chunk → complete', () => {
    const { beginRequest, markStarted, appendChunk, markCompleted } =
      useTranslationStore.getState();
    beginRequest('req-1');
    expect(useTranslationStore.getState().status).toBe('translating');
    expect(useTranslationStore.getState().requestId).toBe('req-1');

    markStarted({ requestId: 'req-1', startedAtMs: 1000 });
    appendChunk({ requestId: 'req-1', delta: 'Hello' });
    appendChunk({ requestId: 'req-1', delta: ', world!' });
    expect(useTranslationStore.getState().output).toBe('Hello, world!');

    markCompleted({ requestId: 'req-1', fullText: 'Hello, world!', durationMs: 123 });
    expect(useTranslationStore.getState().status).toBe('completed');
    expect(useTranslationStore.getState().durationMs).toBe(123);
  });

  it('ignores chunks from stale requestIds', () => {
    const { beginRequest, appendChunk, markCompleted } = useTranslationStore.getState();
    beginRequest('req-A');
    appendChunk({ requestId: 'req-A', delta: 'a' });
    beginRequest('req-B');
    appendChunk({ requestId: 'req-A', delta: 'should-be-ignored' });
    appendChunk({ requestId: 'req-B', delta: 'b' });
    expect(useTranslationStore.getState().output).toBe('b');
    markCompleted({ requestId: 'req-A', fullText: 'discard', durationMs: 50 });
    expect(useTranslationStore.getState().status).toBe('translating');
  });

  it('markCancelled and markError clear requestId', () => {
    const { beginRequest, markCancelled, markError } = useTranslationStore.getState();
    beginRequest('req-C');
    markCancelled({ requestId: 'req-C' });
    expect(useTranslationStore.getState().status).toBe('cancelled');
    expect(useTranslationStore.getState().requestId).toBeNull();

    beginRequest('req-D');
    markError({
      requestId: 'req-D',
      error: { kind: 'OllamaNotRunning' },
    });
    expect(useTranslationStore.getState().status).toBe('error');
    expect(useTranslationStore.getState().error?.kind).toBe('OllamaNotRunning');
    expect(useTranslationStore.getState().requestId).toBeNull();
  });

  it('clearOutput resets output, duration, and error without affecting input', () => {
    const store = useTranslationStore.getState();
    store.setSourceText('keep this');
    store.beginRequest('req-E');
    store.appendChunk({ requestId: 'req-E', delta: 'partial' });
    store.markError({ requestId: 'req-E', error: { kind: 'Internal', message: 'boom' } });
    store.clearOutput();
    const state = useTranslationStore.getState();
    expect(state.sourceText).toBe('keep this');
    expect(state.output).toBe('');
    expect(state.error).toBeNull();
    expect(state.durationMs).toBeNull();
  });

  it('setLocalError surfaces client-side error regardless of requestId', () => {
    const store = useTranslationStore.getState();
    // idle 상태 (requestId=null) 에서도 표시되어야 한다.
    store.setLocalError({ kind: 'InputTooLong', limit: 30000 });
    let state = useTranslationStore.getState();
    expect(state.status).toBe('error');
    expect(state.error).toEqual({ kind: 'InputTooLong', limit: 30000 });
    expect(state.requestId).toBeNull();

    // 진행 중 상태에서 setLocalError 가 호출되면 in-flight 매칭에 의존하지 않고 갱신.
    store.beginRequest('req-X');
    expect(useTranslationStore.getState().status).toBe('translating');
    store.setLocalError({ kind: 'InputTooLong', limit: 30000 });
    state = useTranslationStore.getState();
    expect(state.status).toBe('error');
    expect(state.requestId).toBeNull();
    expect(state.output).toBe('');
  });

  it('markCompleted records translation into recent[] (newest first)', () => {
    const store = useTranslationStore.getState();
    store.setSourceText('첫번째');
    store.beginRequest('req-1');
    store.markCompleted({ requestId: 'req-1', fullText: 'first', durationMs: 100 });
    store.setSourceText('두번째');
    store.beginRequest('req-2');
    store.markCompleted({ requestId: 'req-2', fullText: 'second', durationMs: 120 });

    const recent = useTranslationStore.getState().recent;
    expect(recent).toHaveLength(2);
    expect(recent[0]?.requestId).toBe('req-2');
    expect(recent[0]?.fullText).toBe('second');
    expect(recent[1]?.requestId).toBe('req-1');
  });

  it('recent[] is capped at RECENT_LIMIT', () => {
    const store = useTranslationStore.getState();
    for (let i = 0; i < 7; i++) {
      store.beginRequest(`req-${i}`);
      store.markCompleted({
        requestId: `req-${i}`,
        fullText: `translation-${i}`,
        durationMs: 50,
      });
    }
    const recent = useTranslationStore.getState().recent;
    expect(recent).toHaveLength(5);
    // 최신순 — 마지막에 들어간 req-6 이 첫 항목.
    expect(recent[0]?.requestId).toBe('req-6');
    // 가장 오래된 5번째 — req-2 (req-0, req-1 은 evict).
    expect(recent[4]?.requestId).toBe('req-2');
  });

  it('setIdle returns store to idle and clears output/request/error', () => {
    const store = useTranslationStore.getState();
    store.beginRequest('req-Y');
    store.appendChunk({ requestId: 'req-Y', delta: 'partial' });
    store.markCompleted({ requestId: 'req-Y', fullText: 'partial', durationMs: 99 });
    expect(useTranslationStore.getState().status).toBe('completed');

    store.setIdle();
    const state = useTranslationStore.getState();
    expect(state.status).toBe('idle');
    expect(state.requestId).toBeNull();
    expect(state.output).toBe('');
    expect(state.durationMs).toBeNull();
    expect(state.error).toBeNull();
  });
});
