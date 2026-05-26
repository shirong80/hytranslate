import { create } from 'zustand';

import type { AppError } from '@lib/ipc/errors';

import { getSettings, updateSettings } from './ipc';
import { DEFAULT_SETTINGS, type Settings } from './types';

export interface SettingsState {
  settings: Settings;
  loaded: boolean;
  saving: boolean;
  error: AppError | null;
}

export interface SettingsActions {
  load: () => Promise<void>;
  save: (next: Settings) => Promise<void>;
  /** test/teardown 전용 — 상태를 초기화한다. */
  reset: () => void;
}

const initialState: SettingsState = {
  settings: DEFAULT_SETTINGS,
  loaded: false,
  saving: false,
  error: null,
};

export const useSettingsStore = create<SettingsState & SettingsActions>()((set) => ({
  ...initialState,

  load: async () => {
    set({ error: null });
    try {
      const settings = await getSettings();
      set({ settings, loaded: true });
    } catch (err) {
      set({ error: toAppError(err) });
    }
  },

  save: async (next) => {
    set({ saving: true, error: null });
    try {
      const saved = await updateSettings(next);
      set({ settings: saved, saving: false });
    } catch (err) {
      set({ error: toAppError(err), saving: false });
    }
  },

  reset: () => {
    set(initialState);
  },
}));

function toAppError(err: unknown): AppError {
  if (typeof err === 'object' && err !== null && 'kind' in err) {
    return err as AppError;
  }
  return { kind: 'Internal', message: err instanceof Error ? err.message : String(err) };
}
