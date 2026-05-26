import { create } from 'zustand';

import type { AppError } from '@lib/ipc/errors';

import {
  type RecentTranslation,
  type SourceLanguage,
  type TranslationStatus,
  DEFAULT_MODEL,
  RECENT_LIMIT,
} from './types';

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
  /** in-memory recent translations; Phase 4 에서 SQLite + FTS5 로 대체된다. */
  recent: RecentTranslation[];
}

export interface TranslationActions {
  setSourceText: (text: string) => void;
  setSourceLanguage: (lang: SourceLanguage) => void;
  /** Settings 가 활성 모델을 푸시할 때 호출. translation store 가 settings 를 import 하지 않도록 외부 주입 패턴. */
  setModel: (model: string) => void;
  beginRequest: (requestId: string) => void;
  markStarted: (payload: { requestId: string; startedAtMs: number }) => void;
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
  recent: [],
};

export const useTranslationStore = create<TranslationState & TranslationActions>()((set, get) => ({
  ...initialState,

  setSourceText: (text) => {
    set({ sourceText: text });
  },

  setSourceLanguage: (lang) => {
    set({ sourceLanguage: lang });
  },

  setModel: (model) => {
    set({ model });
  },

  beginRequest: (requestId) => {
    set({
      requestId,
      status: 'translating',
      output: '',
      error: null,
      startedAtMs: null,
      durationMs: null,
    });
  },

  markStarted: ({ requestId, startedAtMs }) => {
    if (get().requestId !== requestId) return;
    set({ startedAtMs, status: 'translating' });
  },

  appendChunk: ({ requestId, delta }) => {
    if (get().requestId !== requestId) return;
    set((state) => ({ output: state.output + delta }));
  },

  markCompleted: ({ requestId, fullText, durationMs }) => {
    if (get().requestId !== requestId) return;
    const { sourceText, sourceLanguage, recent } = get();
    const entry: RecentTranslation = {
      requestId,
      sourceText,
      fullText,
      sourceLanguage,
      durationMs,
      completedAtMs: Date.now(),
    };
    // 새 항목이 가장 앞. RECENT_LIMIT 개 까지만 유지.
    const nextRecent = [entry, ...recent.filter((r) => r.requestId !== requestId)].slice(
      0,
      RECENT_LIMIT,
    );
    set({
      output: fullText,
      durationMs,
      status: 'completed',
      error: null,
      recent: nextRecent,
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
    });
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
    });
  },

  reset: () => {
    set(initialState);
  },
}));
