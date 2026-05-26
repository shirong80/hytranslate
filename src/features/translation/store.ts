import { create } from 'zustand';

import type { AppError } from '@lib/ipc/errors';

import { type SourceLanguage, type TranslationStatus, DEFAULT_MODEL } from './types';

export interface TranslationState {
  sourceText: string;
  sourceLanguage: SourceLanguage;
  model: string;
  output: string;
  status: TranslationStatus;
  error: AppError | null;
  requestId: string | null;
  startedAtMs: number | null;
  durationMs: number | null;
  /** Auto 입력의 backend 감지 결과. 수동 선택이면 null. */
  resolvedLanguage: SourceLanguage | null;
  /** 결과 복사 실패. 1.5초 후 자동 소멸 — UI 가 setTimeout 으로 해제한다. */
  copyError: AppError | null;
}

export interface TranslationActions {
  setSourceText: (text: string) => void;
  setSourceLanguage: (lang: SourceLanguage) => void;
  /** Settings 가 활성 모델을 푸시할 때 호출. translation store 가 settings 를 import 하지 않도록 외부 주입 패턴. */
  setModel: (model: string) => void;
  /** 입력 변화가 감지되어 debounce 대기 중. */
  setTyping: () => void;
  /** runTranslation 직전 짧게 표시 — Auto 입력에서 backend 감지가 끝나기까지의 transient. */
  setDetecting: () => void;
  beginRequest: (requestId: string) => void;
  markStarted: (payload: {
    requestId: string;
    startedAtMs: number;
    resolvedLanguage: SourceLanguage;
  }) => void;
  appendChunk: (payload: { requestId: string; delta: string }) => void;
  markCompleted: (payload: { requestId: string; fullText: string; durationMs: number }) => void;
  markCancelled: (payload: { requestId: string }) => void;
  markError: (payload: { requestId: string; error: AppError }) => void;
  /**
   * client-side 검증 에러를 표시한다. in-flight 요청이 있다면 즉시 무효화한다 —
   * 호출자가 이미 cancelTranslation을 보냈다는 가정. server lifecycle와 분리된
   * action 이므로 requestId 매칭 검사를 거치지 않는다.
   */
  setLocalError: (error: AppError) => void;
  setCopyError: (error: AppError | null) => void;
  clearOutput: () => void;
  setIdle: () => void;
  reset: () => void;
}

const initialState: TranslationState = {
  sourceText: '',
  sourceLanguage: 'Auto',
  model: DEFAULT_MODEL,
  output: '',
  status: 'idle',
  error: null,
  requestId: null,
  startedAtMs: null,
  durationMs: null,
  resolvedLanguage: null,
  copyError: null,
};

export const useTranslationStore = create<TranslationState & TranslationActions>()((set, get) => ({
  ...initialState,

  setSourceText: (text) => {
    set({ sourceText: text });
  },

  setSourceLanguage: (lang) => {
    set({ sourceLanguage: lang, resolvedLanguage: null });
  },

  setModel: (model) => {
    set({ model });
  },

  setTyping: () => {
    set({
      status: 'typing',
      error: null,
      output: '',
      durationMs: null,
      startedAtMs: null,
      resolvedLanguage: null,
    });
  },

  setDetecting: () => {
    set({ status: 'detecting', error: null });
  },

  beginRequest: (requestId) => {
    set((s) => ({
      requestId,
      // code-review v1 follow-up §20 — Auto 입력의 transient `detecting` 을 보존한다.
      // `markStarted` 이벤트가 도착하면 그때 `translating` 으로 전환된다.
      status: s.status === 'detecting' ? 'detecting' : 'translating',
      output: '',
      error: null,
      startedAtMs: null,
      durationMs: null,
      resolvedLanguage: null,
    }));
  },

  markStarted: ({ requestId, startedAtMs, resolvedLanguage }) => {
    if (get().requestId !== requestId) return;
    set({ startedAtMs, status: 'translating', resolvedLanguage });
  },

  appendChunk: ({ requestId, delta }) => {
    if (get().requestId !== requestId) return;
    set((state) => ({ output: state.output + delta }));
  },

  markCompleted: ({ requestId, fullText, durationMs }) => {
    if (get().requestId !== requestId) return;
    set({
      output: fullText,
      durationMs,
      status: 'completed',
      error: null,
    });
  },

  markCancelled: ({ requestId }) => {
    if (get().requestId !== requestId) return;
    set({ status: 'cancelled', requestId: null });
  },

  markError: ({ requestId, error }) => {
    if (get().requestId !== requestId) return;
    set({ status: 'error', error, requestId: null });
  },

  setLocalError: (error) => {
    set({
      status: 'error',
      error,
      requestId: null,
      output: '',
      durationMs: null,
      startedAtMs: null,
      resolvedLanguage: null,
    });
  },

  setCopyError: (error) => {
    set({ copyError: error });
  },

  clearOutput: () => {
    set({ output: '', durationMs: null, error: null });
  },

  setIdle: () => {
    set({
      status: 'idle',
      error: null,
      requestId: null,
      output: '',
      startedAtMs: null,
      durationMs: null,
      resolvedLanguage: null,
    });
  },

  reset: () => {
    set(initialState);
  },
}));
