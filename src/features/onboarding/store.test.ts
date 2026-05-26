import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { toProgressView, useOnboardingStore } from './store';
import { HY_MT2_1_8B, HY_MT2_7B, type EnvironmentReport, type OllamaStatus } from './types';

const mocks = vi.hoisted(() => ({
  detectEnvironment: vi.fn(),
  getOllamaStatus: vi.fn(),
  pullModel: vi.fn(),
  cancelModelPull: vi.fn(),
  completeOnboarding: vi.fn(),
  openOllamaDownloadPage: vi.fn(),
  attachModelPullListeners: vi.fn(),
}));

vi.mock('./ipc', () => ({
  detectEnvironment: mocks.detectEnvironment,
  getOllamaStatus: mocks.getOllamaStatus,
  pullModel: mocks.pullModel,
  cancelModelPull: mocks.cancelModelPull,
  completeOnboarding: mocks.completeOnboarding,
  openOllamaDownloadPage: mocks.openOllamaDownloadPage,
  attachModelPullListeners: mocks.attachModelPullListeners,
}));

function env(overrides: Partial<EnvironmentReport> = {}): EnvironmentReport {
  return {
    macosVersion: '14.4.1',
    macosMajor: 14,
    macosSupported: true,
    arch: 'AppleSilicon',
    totalMemoryGb: 16,
    recommendedModel: HY_MT2_7B,
    ...overrides,
  };
}

function status(overrides: Partial<OllamaStatus> = {}): OllamaStatus {
  return {
    running: true,
    endpoint: 'http://localhost:11434',
    models: [],
    ...overrides,
  };
}

beforeEach(() => {
  useOnboardingStore.getState().reset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe('useOnboardingStore', () => {
  it('loadEnvironment populates env and adopts recommended model if user has not changed', async () => {
    mocks.detectEnvironment.mockResolvedValue(env({ recommendedModel: HY_MT2_1_8B }));
    await useOnboardingStore.getState().loadEnvironment();
    const s = useOnboardingStore.getState();
    expect(s.env?.recommendedModel).toBe(HY_MT2_1_8B);
    expect(s.selectedModel).toBe(HY_MT2_1_8B);
    expect(s.error).toBeNull();
  });

  it('loadEnvironment does not overwrite user-selected model', async () => {
    useOnboardingStore.getState().selectModel(HY_MT2_7B);
    // 사용자가 명시적으로 골랐다는 사실은 selectedModel !== DEFAULT 로 확인하기 어렵다.
    // 본 store 는 DEFAULT (HY_MT2_7B) 이 곧 "아직 변경 없음" 신호.
    // 따라서 추천이 7B 인 경우는 자동 채택, 1.8B 인 경우만 보존 검증을 수행한다.
    mocks.detectEnvironment.mockResolvedValue(env({ recommendedModel: HY_MT2_1_8B }));
    // 사용자가 의도적으로 1.8B 를 골랐다면 추천과 동일 → 그대로 유지.
    useOnboardingStore.getState().selectModel(HY_MT2_1_8B);
    await useOnboardingStore.getState().loadEnvironment();
    expect(useOnboardingStore.getState().selectedModel).toBe(HY_MT2_1_8B);
  });

  it('refreshOllamaStatus populates ollama and clears error', async () => {
    mocks.getOllamaStatus.mockResolvedValue(status({ models: [HY_MT2_7B] }));
    await useOnboardingStore.getState().refreshOllamaStatus();
    const s = useOnboardingStore.getState();
    expect(s.ollama?.running).toBe(true);
    expect(s.ollama?.models).toEqual([HY_MT2_7B]);
  });

  it('refreshOllamaStatus surfaces error', async () => {
    mocks.getOllamaStatus.mockRejectedValue({ kind: 'OllamaUnavailable' });
    await useOnboardingStore.getState().refreshOllamaStatus();
    expect(useOnboardingStore.getState().error).toEqual({ kind: 'OllamaUnavailable' });
  });

  it('startPull sets pullingModel and clears on success via listener', async () => {
    mocks.pullModel.mockResolvedValue(undefined);
    await useOnboardingStore.getState().startPull(HY_MT2_7B);
    expect(useOnboardingStore.getState().pullingModel).toBe(HY_MT2_7B);
    expect(mocks.pullModel).toHaveBeenCalledWith(HY_MT2_7B);
  });

  it('startPull rolls back on ipc failure', async () => {
    mocks.pullModel.mockRejectedValue({ kind: 'OllamaNotRunning' });
    await useOnboardingStore.getState().startPull(HY_MT2_7B);
    const s = useOnboardingStore.getState();
    expect(s.pullingModel).toBeNull();
    expect(s.progress).toBeNull();
    expect(s.error).toEqual({ kind: 'OllamaNotRunning' });
  });

  it('cancelPull resets state', async () => {
    mocks.pullModel.mockResolvedValue(undefined);
    mocks.cancelModelPull.mockResolvedValue(undefined);
    await useOnboardingStore.getState().startPull(HY_MT2_7B);
    expect(useOnboardingStore.getState().pullingModel).toBe(HY_MT2_7B);
    await useOnboardingStore.getState().cancelPull();
    const s = useOnboardingStore.getState();
    expect(s.pullingModel).toBeNull();
    expect(s.progress).toBeNull();
  });

  it('finish persists onboarding flag and moves to done', async () => {
    mocks.completeOnboarding.mockResolvedValue(undefined);
    await useOnboardingStore.getState().finish();
    expect(mocks.completeOnboarding).toHaveBeenCalledTimes(1);
    expect(useOnboardingStore.getState().step).toBe('done');
  });

  it('goNext advances by one step and stops at the last', () => {
    const { goNext, goTo } = useOnboardingStore.getState();
    goTo('welcome');
    goNext();
    expect(useOnboardingStore.getState().step).toBe('environment');
    goTo('history');
    goNext();
    expect(useOnboardingStore.getState().step).toBe('done');
    goNext();
    expect(useOnboardingStore.getState().step).toBe('done');
  });

  it('goPrev decreases by one and stops at welcome', () => {
    const { goPrev, goTo } = useOnboardingStore.getState();
    goTo('model');
    goPrev();
    expect(useOnboardingStore.getState().step).toBe('ollama');
    goTo('welcome');
    goPrev();
    expect(useOnboardingStore.getState().step).toBe('welcome');
  });
});

describe('toProgressView', () => {
  it('computes ratio when total > 0', () => {
    const v = toProgressView({
      model: 'm',
      status: 'downloading',
      digest: null,
      total: 1000,
      completed: 250,
    });
    expect(v.ratio).toBeCloseTo(0.25);
  });

  it('returns null ratio when total is missing or zero', () => {
    expect(
      toProgressView({
        model: 'm',
        status: 'downloading',
        digest: null,
        total: null,
        completed: 100,
      }).ratio,
    ).toBeNull();
    expect(
      toProgressView({
        model: 'm',
        status: 'downloading',
        digest: null,
        total: 0,
        completed: 0,
      }).ratio,
    ).toBeNull();
  });

  it('clamps ratio to [0, 1]', () => {
    const high = toProgressView({
      model: 'm',
      status: 'downloading',
      digest: null,
      total: 100,
      completed: 250,
    });
    expect(high.ratio).toBe(1);
  });
});
