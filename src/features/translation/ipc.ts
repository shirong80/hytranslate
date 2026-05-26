import { invoke, listen } from '@lib/ipc/client';
import {
  TRANSLATION_CANCELLED,
  TRANSLATION_CHUNK,
  TRANSLATION_COMPLETED,
  TRANSLATION_ERROR,
  TRANSLATION_STARTED,
} from '@lib/ipc/events';

import type {
  CancelledPayload,
  ChunkPayload,
  CompletedPayload,
  ErrorPayload,
  StartedPayload,
  TranslateRequest,
} from './types';

export interface TranslationListeners {
  onStarted?: (payload: StartedPayload) => void;
  onChunk?: (payload: ChunkPayload) => void;
  onCompleted?: (payload: CompletedPayload) => void;
  onCancelled?: (payload: CancelledPayload) => void;
  onError?: (payload: ErrorPayload) => void;
}

export type UnlistenAll = () => void;

export async function translateStream(request: TranslateRequest): Promise<void> {
  return invoke<void>('translate_stream', { request });
}

export async function cancelTranslation(requestId: string): Promise<void> {
  return invoke<void>('cancel_translation', { requestId });
}

export async function attachTranslationListeners(
  listeners: TranslationListeners,
): Promise<UnlistenAll> {
  const unlisteners = await Promise.all([
    listeners.onStarted
      ? listen<StartedPayload>(TRANSLATION_STARTED, listeners.onStarted)
      : Promise.resolve(() => {}),
    listeners.onChunk
      ? listen<ChunkPayload>(TRANSLATION_CHUNK, listeners.onChunk)
      : Promise.resolve(() => {}),
    listeners.onCompleted
      ? listen<CompletedPayload>(TRANSLATION_COMPLETED, listeners.onCompleted)
      : Promise.resolve(() => {}),
    listeners.onCancelled
      ? listen<CancelledPayload>(TRANSLATION_CANCELLED, listeners.onCancelled)
      : Promise.resolve(() => {}),
    listeners.onError
      ? listen<ErrorPayload>(TRANSLATION_ERROR, listeners.onError)
      : Promise.resolve(() => {}),
  ]);

  return () => {
    for (const off of unlisteners) {
      off();
    }
  };
}
