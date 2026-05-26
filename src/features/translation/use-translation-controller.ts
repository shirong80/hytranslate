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
  const setLocalError = useTranslationStore((s) => s.setLocalError);
  const setIdle = useTranslationStore((s) => s.setIdle);

  const debounceTimer = useRef<number | null>(null);
  const inFlightRef = useRef<string | null>(null);

  /** in-flight 요청이 있으면 백엔드 취소를 보내고 ref 를 비운다. */
  const cancelInFlight = useCallback(async () => {
    const previousId = inFlightRef.current;
    if (!previousId) return;
    inFlightRef.current = null;
    try {
      await cancelTranslation(previousId);
    } catch {
      // 이미 종료된 요청일 수 있음. 무시.
    }
  }, []);

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
      await cancelInFlight();
      setIdle();
      return;
    }
    // 길이 초과는 client-side 검증. in-flight 요청을 먼저 취소하여 stale chunk가
    // 새 상태에 누적되지 않게 한다.
    if ([...text].length > MAIN_INPUT_LIMIT) {
      await cancelInFlight();
      setLocalError({ kind: 'InputTooLong', limit: MAIN_INPUT_LIMIT });
      return;
    }

    await cancelInFlight();

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
  }, [beginRequest, cancelInFlight, markError, setIdle, setLocalError]);

  useEffect(() => {
    if (debounceTimer.current) {
      window.clearTimeout(debounceTimer.current);
      debounceTimer.current = null;
    }
    if (sourceText.trim().length === 0) {
      void cancelInFlight().then(() => setIdle());
      return;
    }
    debounceTimer.current = window.setTimeout(() => {
      runTranslation().catch(() => {
        // 이미 markError로 매핑됨
      });
    }, debounceMs);

    return () => {
      if (debounceTimer.current) {
        window.clearTimeout(debounceTimer.current);
        debounceTimer.current = null;
      }
    };
  }, [sourceText, sourceLanguage, model, debounceMs, runTranslation, cancelInFlight, setIdle]);

  const retranslateImmediately = useCallback(() => {
    if (debounceTimer.current) {
      window.clearTimeout(debounceTimer.current);
      debounceTimer.current = null;
    }
    void runTranslation();
  }, [runTranslation]);

  const cancelCurrent = useCallback(() => {
    void cancelInFlight();
  }, [cancelInFlight]);

  return { runImmediately: retranslateImmediately, cancelCurrent, currentRequestId: requestId };
}
