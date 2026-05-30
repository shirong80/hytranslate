import { useCallback, useEffect, useRef } from 'react';

import { saveTranslationRecord } from '@features/history/ipc';
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
  /** 입력 cap. 메인=30k(MAIN_INPUT_LIMIT), 팝업/메뉴바=5k(POPUP_INPUT_LIMIT). */
  inputLimit?: number;
}

export function useTranslationController(options: UseTranslationControllerOptions = {}) {
  const debounceMs = options.debounceMs ?? DEBOUNCE_MS;
  const inputLimit = options.inputLimit ?? MAIN_INPUT_LIMIT;

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
  const setTyping = useTranslationStore((s) => s.setTyping);
  const setDetecting = useTranslationStore((s) => s.setDetecting);
  const setSourceText = useTranslationStore((s) => s.setSourceText);

  const debounceTimer = useRef<number | null>(null);
  const inFlightRef = useRef<string | null>(null);

  /** in-flight 요청이 있으면 백엔드 취소를 보내고 ref 를 비운다. backend cancel 은 idempotent. */
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
    if ([...text].length > inputLimit) {
      await cancelInFlight();
      setLocalError({ kind: 'InputTooLong', limit: inputLimit });
      return;
    }

    // retranslateImmediately() 경로 (Cmd+Enter / 다시 번역) — 입력 deps 가 그대로일 때도
    // 기존 in-flight 가 새 요청과 병행되지 않도록 방어선 유지.
    await cancelInFlight();

    // Auto 입력에서만 detecting transient. 수동 선택이면 backend 가 detect 자체를
    // 건너뛰므로 "언어 감지 중…" 을 잠깐 보여줄 이유가 없다.
    if (useTranslationStore.getState().sourceLanguage === 'Auto') {
      setDetecting();
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
  }, [beginRequest, cancelInFlight, inputLimit, markError, setDetecting, setIdle, setLocalError]);

  useEffect(() => {
    if (debounceTimer.current) {
      window.clearTimeout(debounceTimer.current);
      debounceTimer.current = null;
    }
    // code-review v1 follow-up §15 — effect-local cancelled flag.
    // 입력이 빠르게 바뀌면 이전 effect 의 `cancelInFlight().then(...)` continuation 이
    // cleanup 이후 늦게 실행돼 stale timer 를 다시 예약할 수 있다. flag 로 그 continuation
    // 을 차단해 마지막 effect 만 timer 를 예약하도록 한다.
    let cancelled = false;

    // Critical 2 — 입력 / 언어 / 모델 변경 즉시 in-flight 취소.
    // stale 결과가 새 입력에 덮어쓰이거나 DB 에 저장되는 race 를 차단한다.
    void cancelInFlight().then(() => {
      if (cancelled) return;
      if (sourceText.trim().length === 0) {
        setIdle();
        return;
      }
      setTyping();
      debounceTimer.current = window.setTimeout(() => {
        runTranslation().catch(() => {
          // 이미 markError로 매핑됨
        });
      }, debounceMs);
    });

    return () => {
      cancelled = true;
      if (debounceTimer.current) {
        window.clearTimeout(debounceTimer.current);
        debounceTimer.current = null;
      }
    };
  }, [
    sourceText,
    sourceLanguage,
    model,
    debounceMs,
    runTranslation,
    cancelInFlight,
    setIdle,
    setTyping,
  ]);

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

  /**
   * Cmd+Enter — 완료된 번역을 이력에 저장하고 입력/출력을 전체 초기화한다.
   * - 결과가 준비되지 않았으면 (번역 중·빈 출력 등) no-op.
   * - 저장이 실패해도 입력은 비운다 — 이력 손실은 비치명적, UI 흐름을 막지 않는다.
   * - `save_history` 게이팅은 백엔드가 담당 (OFF 면 INSERT 없이 Ok → 여기선 그대로 초기화).
   */
  const saveAndClear = useCallback(async () => {
    const s = useTranslationStore.getState();
    if (s.status !== 'completed' || s.output.trim().length === 0) return;
    try {
      await saveTranslationRecord({
        id: s.requestId ?? generateRequestId(),
        sourceText: s.sourceText,
        sourceLanguage: s.resolvedLanguage ?? s.sourceLanguage,
        translatedText: s.output,
        model: s.model,
        durationMs: s.durationMs ?? 0,
      });
    } catch {
      // 이력 저장 실패는 무시 — finally 에서 입력을 비워 흐름을 보존한다.
    } finally {
      setSourceText('');
      setIdle();
    }
  }, [setSourceText, setIdle]);

  return {
    runImmediately: retranslateImmediately,
    saveAndClear,
    cancelCurrent,
    currentRequestId: requestId,
  };
}
