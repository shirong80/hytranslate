// `crate::environment::EnvironmentReport` / `commands::onboarding::OllamaStatus` 와 1:1 mirror.

import type { AppError } from '@lib/ipc/errors';

export const ARCHES = ['AppleSilicon', 'Intel', 'Unknown'] as const;
export type Arch = (typeof ARCHES)[number];

export interface EnvironmentReport {
  macosVersion: string;
  macosMajor: number;
  macosSupported: boolean;
  arch: Arch;
  totalMemoryGb: number;
  recommendedModel: string;
}

export interface OllamaStatus {
  installed: boolean;
  running: boolean;
  endpoint: string;
  models: string[];
}

export interface PullStartedPayload {
  model: string;
}

export interface PullProgressPayload {
  model: string;
  status: string;
  digest: string | null;
  total: number | null;
  completed: number | null;
}

export interface PullCompletedPayload {
  model: string;
}

export interface PullErrorPayload {
  model: string;
  error: AppError;
}

export const HY_MT2_7B = 'hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M' as const;
export const HY_MT2_1_8B = 'hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M' as const;

export type OnboardingStep =
  | 'welcome'
  | 'environment'
  | 'ollama'
  | 'model'
  | 'permissions'
  | 'history'
  | 'done';

export const ONBOARDING_STEPS: OnboardingStep[] = [
  'welcome',
  'environment',
  'ollama',
  'model',
  'permissions',
  'history',
  'done',
];
