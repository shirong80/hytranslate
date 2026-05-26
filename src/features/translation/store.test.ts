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
    // idle 에서 beginRequest 호출하면 detecting 이 아니므로 translating 으로.
    expect(useTranslationStore.getState().status).toBe('translating');
    expect(useTranslationStore.getState().requestId).toBe('req-1');

    markStarted({ requestId: 'req-1', startedAtMs: 1000, resolvedLanguage: 'Korean' });
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

  it('setTyping / setDetecting transition through Major 8 states', () => {
    const store = useTranslationStore.getState();
    store.setTyping();
    expect(useTranslationStore.getState().status).toBe('typing');
    expect(useTranslationStore.getState().output).toBe('');
    expect(useTranslationStore.getState().resolvedLanguage).toBeNull();

    store.setDetecting();
    expect(useTranslationStore.getState().status).toBe('detecting');
  });

  it('beginRequest preserves detecting transient (Medium 1)', () => {
    const store = useTranslationStore.getState();
    store.setDetecting();
    store.beginRequest('req-detect');
    expect(useTranslationStore.getState().status).toBe('detecting');
    // markStarted 가 도착하면 translating 으로.
    store.markStarted({
      requestId: 'req-detect',
      startedAtMs: 1,
      resolvedLanguage: 'Korean',
    });
    expect(useTranslationStore.getState().status).toBe('translating');
  });

  it('markStarted stores resolvedLanguage for the matching requestId', () => {
    const store = useTranslationStore.getState();
    store.beginRequest('req-Z');
    store.markStarted({
      requestId: 'req-Z',
      startedAtMs: 100,
      resolvedLanguage: 'ChineseSimplified',
    });
    expect(useTranslationStore.getState().resolvedLanguage).toBe('ChineseSimplified');
    // 다른 requestId 는 무시.
    store.markStarted({
      requestId: 'stale',
      startedAtMs: 999,
      resolvedLanguage: 'Korean',
    });
    expect(useTranslationStore.getState().resolvedLanguage).toBe('ChineseSimplified');
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
