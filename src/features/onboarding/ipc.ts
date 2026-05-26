import { invoke, listen } from '@lib/ipc/client';
import {
  MODEL_PULL_COMPLETED,
  MODEL_PULL_ERROR,
  MODEL_PULL_PROGRESS,
  MODEL_PULL_STARTED,
} from '@lib/ipc/events';

import type {
  EnvironmentReport,
  OllamaStatus,
  PullCompletedPayload,
  PullErrorPayload,
  PullProgressPayload,
  PullStartedPayload,
} from './types';

export interface ModelPullListeners {
  onStarted?: (payload: PullStartedPayload) => void;
  onProgress?: (payload: PullProgressPayload) => void;
  onCompleted?: (payload: PullCompletedPayload) => void;
  onError?: (payload: PullErrorPayload) => void;
}

export type UnlistenAll = () => void;

export async function detectEnvironment(): Promise<EnvironmentReport> {
  return invoke<EnvironmentReport>('detect_environment');
}

export async function getOllamaStatus(): Promise<OllamaStatus> {
  return invoke<OllamaStatus>('get_ollama_status');
}

export async function pullModel(model: string): Promise<void> {
  return invoke<void>('pull_model', { request: { model } });
}

export async function cancelModelPull(model: string): Promise<void> {
  return invoke<void>('cancel_model_pull', { request: { model } });
}

export async function completeOnboarding(): Promise<void> {
  return invoke<void>('complete_onboarding');
}

export async function openOllamaDownloadPage(): Promise<void> {
  return invoke<void>('open_ollama_download_page');
}

export async function attachModelPullListeners(
  listeners: ModelPullListeners,
): Promise<UnlistenAll> {
  const unlisteners = await Promise.all([
    listeners.onStarted
      ? listen<PullStartedPayload>(MODEL_PULL_STARTED, listeners.onStarted)
      : Promise.resolve(() => {}),
    listeners.onProgress
      ? listen<PullProgressPayload>(MODEL_PULL_PROGRESS, listeners.onProgress)
      : Promise.resolve(() => {}),
    listeners.onCompleted
      ? listen<PullCompletedPayload>(MODEL_PULL_COMPLETED, listeners.onCompleted)
      : Promise.resolve(() => {}),
    listeners.onError
      ? listen<PullErrorPayload>(MODEL_PULL_ERROR, listeners.onError)
      : Promise.resolve(() => {}),
  ]);
  return () => {
    for (const off of unlisteners) {
      off();
    }
  };
}
