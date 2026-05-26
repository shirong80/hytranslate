import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event';

export async function invoke<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(command, payload);
}

export async function listen<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return tauriListen<T>(event, ({ payload }) => handler(payload));
}
