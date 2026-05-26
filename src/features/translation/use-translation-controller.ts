import { useCallback, useEffect, useRef } from 'react';

import { isAppError } from '@lib/ipc/errors';

import { attachTranslationListeners, cancelTranslation, translateStream } from './ipc';
import { useTranslationStore } from './store';
import { MAIN_INPUT_LIMIT } from './types';

const DEBOUNCE_MS = 500;

function generateRequestId(): string {
  return globalThis.crypto.randomUUID();
}

interface UseTranslationControllerOptions {
  debounceMs?: number;
}

export function useTranslationController(options: UseTranslationControllerOptions = {}) {
  const debounceMs = options.debounceMs ?? DEBOUNCE_MS;

  const sourceText = useTranslationStore((s) => s.sourceText);
  const sourceLanguage = useTranslationStore((s) => s.sourceLanguage);
  const model = useTranslationStore((s) => s.model);
  const requestId = useTranslationStore((s) => s.requestId);

  const beginRequest = useTranslationStore((s) => s.beginRequest);
  const markStarted = useTranslationStore((s) => s.markStarted);
  const appendChunk = useTranslationStore((s) => s.appendChunk);
  const markCompleted = useTranslationStore((s) => s.markCompleted);
  const markCancelled = useTranslationStore((s) => s.markCancelled);
  const markError = useTranslationStore((s) => s.markError);
  const clearOutput = useTranslationStore((s) => s.clearOutput);

  const debounceTimer = useRef<number | null>(null);
  const inFlightRef = useRef<string | null>(null);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    let cancelled = false;

    attachTranslationListeners({
      onStarted: (p) => markStarted(p),
      onChunk: (p) => appendChunk(p),
      onCompleted: (p) => {
        markCompleted(p);
        if (inFlightRef.current === p.requestId) inFlightRef.current = null;
      },
      onCancelled: (p) => {
        markCancelled(p);
        if (inFlightRef.current === p.requestId) inFlightRef.current = null;
      },
      onError: (p) => {
        markError(p);
        if (inFlightRef.current === p.requestId) inFlightRef.current = null;
      },
    }).then((off) => {
      if (cancelled) {
        off();
      } else {
        cleanup = off;
      }
    });

    return () => {
      cancelled = true;
      cleanup?.();
    };
  }, [markStarted, appendChunk, markCompleted, markCancelled, markError]);

  const runTranslation = useCallback(async () => {
    const text = useTranslationStore.getState().sourceText;
    if (text.trim().length === 0) {
      clearOutput();
      return;
    }
    if ([...text].length > MAIN_INPUT_LIMIT) {
      markError({
        requestId: 'local',
        error: { kind: 'InputTooLong', limit: MAIN_INPUT_LIMIT },
      });
      return;
    }

    const previousId = inFlightRef.current;
    if (previousId) {
      try {
        await cancelTranslation(previousId);
      } catch {
        // 무시: 이미 종료된 요청일 수 있음
      }
    }

    const newId = generateRequestId();
    inFlightRef.current = newId;
    beginRequest(newId);

    try {
      await translateStream({
        sourceText: text,
        sourceLanguage: useTranslationStore.getState().sourceLanguage,
        model: useTranslationStore.getState().model,
        requestId: newId,
      });
    } catch (err) {
      if (isAppError(err)) {
        markError({ requestId: newId, error: err });
      } else {
        markError({
          requestId: newId,
          error: { kind: 'Internal', message: err instanceof Error ? err.message : String(err) },
        });
      }
      if (inFlightRef.current === newId) inFlightRef.current = null;
    }
  }, [beginRequest, clearOutput, markError]);

  useEffect(() => {
    if (debounceTimer.current) {
      window.clearTimeout(debounceTimer.current);
      debounceTimer.current = null;
    }
    if (sourceText.trim().length === 0) {
      if (inFlightRef.current) {
        cancelTranslation(inFlightRef.current).catch(() => {
          // ignore
        });
        inFlightRef.current = null;
      }
      clearOutput();
      return;
    }
    debounceTimer.current = window.setTimeout(() => {
      runTranslation().catch(() => {
        // already mapped to markError above
      });
    }, debounceMs);

    return () => {
      if (debounceTimer.current) {
        window.clearTimeout(debounceTimer.current);
        debounceTimer.current = null;
      }
    };
  }, [sourceText, sourceLanguage, model, debounceMs, runTranslation, clearOutput]);

  const retranslateImmediately = useCallback(() => {
    if (debounceTimer.current) {
      window.clearTimeout(debounceTimer.current);
      debounceTimer.current = null;
    }
    void runTranslation();
  }, [runTranslation]);

  const cancelCurrent = useCallback(() => {
    if (inFlightRef.current) {
      void cancelTranslation(inFlightRef.current);
    }
  }, []);

  return { runImmediately: retranslateImmediately, cancelCurrent, currentRequestId: requestId };
}
