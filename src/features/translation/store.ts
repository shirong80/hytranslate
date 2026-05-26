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
}

export interface TranslationActions {
  setSourceText: (text: string) => void;
  setSourceLanguage: (lang: SourceLanguage) => void;
  beginRequest: (requestId: string) => void;
  markStarted: (payload: { requestId: string; startedAtMs: number }) => void;
  appendChunk: (payload: { requestId: string; delta: string }) => void;
  markCompleted: (payload: { requestId: string; fullText: string; durationMs: number }) => void;
  markCancelled: (payload: { requestId: string }) => void;
  markError: (payload: { requestId: string; error: AppError }) => void;
  clearOutput: () => void;
  reset: () => void;
}

const initialState: TranslationState = {
  sourceText: '',
  sourceLanguage: 'Korean',
  model: DEFAULT_MODEL,
  output: '',
  status: 'idle',
  error: null,
  requestId: null,
  startedAtMs: null,
  durationMs: null,
};

export const useTranslationStore = create<TranslationState & TranslationActions>()((set, get) => ({
  ...initialState,

  setSourceText: (text) => {
    set({ sourceText: text });
  },

  setSourceLanguage: (lang) => {
    set({ sourceLanguage: lang });
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

  clearOutput: () => {
    set({ output: '', durationMs: null, error: null });
  },

  reset: () => {
    set(initialState);
  },
}));
