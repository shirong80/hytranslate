import { invoke } from '@lib/ipc/client';

import type { Settings } from './types';

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>('update_settings', { settings });
}
