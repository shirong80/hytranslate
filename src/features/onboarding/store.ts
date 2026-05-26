import { create } from 'zustand';

import { isAppError, type AppError } from '@lib/ipc/errors';

import {
  attachModelPullListeners,
  cancelModelPull,
  completeOnboarding as completeOnboardingIpc,
  detectEnvironment,
  getOllamaStatus,
  pullModel as pullModelIpc,
  tryStartOllama as tryStartOllamaIpc,
} from './ipc';
import type { EnvironmentReport, OllamaStatus, OnboardingStep, PullProgressPayload } from './types';
import { HY_MT2_7B, ONBOARDING_STEPS } from './types';

export interface PullProgressView {
  status: string;
  total: number | null;
  completed: number | null;
  /** 0..=1, total 이 미정이면 null. */
  ratio: number | null;
}

export interface OnboardingState {
  step: OnboardingStep;
  env: EnvironmentReport | null;
  ollama: OllamaStatus | null;
  /** 사용자가 선택한 모델. env 도착 시 추천값으로 초기화. */
  selectedModel: string;
  /** 진행 중인 pull 의 model id. null 이면 idle. */
  pullingModel: string | null;
  progress: PullProgressView | null;
  /** completed pull 한 번이라도 성공한 모델 ids. */
  installedSinceStart: string[];
  loading: boolean;
  error: AppError | null;
}

export interface OnboardingActions {
  goTo: (step: OnboardingStep) => void;
  goNext: () => void;
  goPrev: () => void;
  loadEnvironment: () => Promise<void>;
  refreshOllamaStatus: () => Promise<void>;
  tryStartOllama: () => Promise<void>;
  selectModel: (model: string) => void;
  startPull: (model: string) => Promise<void>;
  cancelPull: () => Promise<void>;
  finish: () => Promise<void>;
  /** 백엔드 model-pull 이벤트 핸들러. App 부트스트랩에서 한 번 호출. */
  bindEventListeners: () => Promise<() => void>;
  reset: () => void;
}

const initialState: OnboardingState = {
  step: 'welcome',
  env: null,
  ollama: null,
  selectedModel: HY_MT2_7B,
  pullingModel: null,
  progress: null,
  installedSinceStart: [],
  loading: false,
  error: null,
};

export const useOnboardingStore = create<OnboardingState & OnboardingActions>()((set, get) => ({
  ...initialState,

  goTo: (step) => set({ step }),

  goNext: () => {
    const idx = ONBOARDING_STEPS.indexOf(get().step);
    const next = ONBOARDING_STEPS[Math.min(idx + 1, ONBOARDING_STEPS.length - 1)];
    if (next) set({ step: next });
  },

  goPrev: () => {
    const idx = ONBOARDING_STEPS.indexOf(get().step);
    const prev = ONBOARDING_STEPS[Math.max(idx - 1, 0)];
    if (prev) set({ step: prev });
  },

  loadEnvironment: async () => {
    set({ loading: true, error: null });
    try {
      const env = await detectEnvironment();
      set((s) => ({
        env,
        // 사용자가 model 화면에서 명시적으로 바꾸기 전까지 추천값을 기본 선택.
        selectedModel: s.selectedModel === HY_MT2_7B ? env.recommendedModel : s.selectedModel,
        loading: false,
      }));
    } catch (err) {
      set({ error: toAppError(err), loading: false });
    }
  },

  refreshOllamaStatus: async () => {
    set({ loading: true, error: null });
    try {
      const ollama = await getOllamaStatus();
      set({ ollama, loading: false });
    } catch (err) {
      set({ error: toAppError(err), loading: false });
    }
  },

  tryStartOllama: async () => {
    set({ loading: true, error: null });
    try {
      await tryStartOllamaIpc();
    } catch (err) {
      set({ error: toAppError(err), loading: false });
      return;
    }
    // Ollama 가 데몬을 띄우기까지 잠시 시간이 필요. 짧은 backoff 후 status 재조회.
    await new Promise((resolve) => setTimeout(resolve, 800));
    try {
      const ollama = await getOllamaStatus();
      set({ ollama, loading: false });
    } catch (err) {
      set({ error: toAppError(err), loading: false });
    }
  },

  selectModel: (model) => set({ selectedModel: model, error: null }),

  startPull: async (model) => {
    set({
      pullingModel: model,
      progress: { status: 'requesting', total: null, completed: null, ratio: null },
      error: null,
    });
    try {
      await pullModelIpc(model);
    } catch (err) {
      set({
        pullingModel: null,
        progress: null,
        error: toAppError(err),
      });
    }
  },

  cancelPull: async () => {
    const m = get().pullingModel;
    if (!m) return;
    try {
      await cancelModelPull(m);
    } catch {
      // Cancel 실패는 무시 — 백엔드 token 은 이미 fired.
    }
    set({ pullingModel: null, progress: null });
  },

  finish: async () => {
    const { selectedModel } = get();
    set({ loading: true, error: null });
    try {
      await completeOnboardingIpc(selectedModel);
      set({ step: 'done', loading: false });
    } catch (err) {
      set({ error: toAppError(err), loading: false });
    }
  },

  bindEventListeners: async () => {
    return attachModelPullListeners({
      onStarted: ({ model }) => {
        if (get().pullingModel === model) {
          set({
            progress: { status: 'started', total: null, completed: null, ratio: null },
          });
        }
      },
      onProgress: (payload) => {
        if (get().pullingModel !== payload.model) return;
        set({ progress: toProgressView(payload) });
      },
      onCompleted: ({ model }) => {
        if (get().pullingModel !== model) return;
        set((s) => ({
          pullingModel: null,
          progress: null,
          installedSinceStart: s.installedSinceStart.includes(model)
            ? s.installedSinceStart
            : [...s.installedSinceStart, model],
        }));
      },
      onError: ({ model, error }) => {
        if (get().pullingModel !== model) return;
        set({ pullingModel: null, progress: null, error });
      },
    });
  },

  reset: () => set(initialState),
}));

export function toProgressView(payload: PullProgressPayload): PullProgressView {
  const ratio =
    payload.total && payload.total > 0 && payload.completed !== null
      ? Math.max(0, Math.min(1, payload.completed / payload.total))
      : null;
  return {
    status: payload.status,
    total: payload.total,
    completed: payload.completed,
    ratio,
  };
}

function toAppError(err: unknown): AppError {
  if (isAppError(err)) return err;
  return { kind: 'Internal', message: err instanceof Error ? err.message : String(err) };
}
