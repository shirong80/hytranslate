import { beforeEach, describe, expect, it, vi } from 'vitest';

import { invoke } from '@lib/ipc/client';

import { useSettingsStore } from './store';
import { DEFAULT_SETTINGS } from './types';

vi.mock('@lib/ipc/client', () => ({
  invoke: vi.fn(),
}));

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe('settings store', () => {
  beforeEach(() => {
    useSettingsStore.getState().reset();
    mockInvoke.mockReset();
  });

  it('starts with default settings, not loaded', () => {
    const state = useSettingsStore.getState();
    expect(state.settings).toEqual(DEFAULT_SETTINGS);
    expect(state.loaded).toBe(false);
    expect(state.error).toBeNull();
  });

  it('load() populates from backend get_settings', async () => {
    const backendValue = { ...DEFAULT_SETTINGS, theme: 'Dark' as const };
    mockInvoke.mockResolvedValueOnce(backendValue);
    await useSettingsStore.getState().load();
    const state = useSettingsStore.getState();
    expect(state.settings).toEqual(backendValue);
    expect(state.loaded).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('get_settings');
  });

  it('save() forwards to update_settings and stores response', async () => {
    const next = { ...DEFAULT_SETTINGS, activeModel: 'custom-model' };
    mockInvoke.mockResolvedValueOnce(next);
    await useSettingsStore.getState().save(next);
    const state = useSettingsStore.getState();
    expect(state.settings).toEqual(next);
    expect(state.saving).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith('update_settings', { settings: next });
  });

  it('save() surfaces AppError from backend', async () => {
    mockInvoke.mockRejectedValueOnce({ kind: 'NetworkBlocked' });
    await useSettingsStore.getState().save({
      ...DEFAULT_SETTINGS,
      ollamaEndpoint: 'http://evil.example.com',
    });
    const state = useSettingsStore.getState();
    expect(state.error?.kind).toBe('NetworkBlocked');
    expect(state.saving).toBe(false);
  });

  it('load() surfaces AppError from backend', async () => {
    mockInvoke.mockRejectedValueOnce({ kind: 'Internal', message: 'disk fail' });
    await useSettingsStore.getState().load();
    const state = useSettingsStore.getState();
    expect(state.error?.kind).toBe('Internal');
    expect(state.loaded).toBe(false);
  });
});
