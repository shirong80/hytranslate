import { beforeEach, describe, expect, it } from 'vitest';

import { useTranslationStore } from './store';

const reset = () => useTranslationStore.getState().reset();

describe('translation store', () => {
  beforeEach(() => {
    reset();
  });

  it('starts idle with default Korean source language', () => {
    const state = useTranslationStore.getState();
    expect(state.status).toBe('idle');
    expect(state.sourceLanguage).toBe('Korean');
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
});
