import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@lib/ipc/client', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@lib/ipc/client';

import { getSettings, updateSettings } from './ipc';
import { DEFAULT_SETTINGS } from './types';

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe('settings ipc', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it('getSettings invokes get_settings with no payload', async () => {
    mockInvoke.mockResolvedValueOnce(DEFAULT_SETTINGS);
    const result = await getSettings();
    expect(result).toEqual(DEFAULT_SETTINGS);
    expect(mockInvoke).toHaveBeenCalledWith('get_settings');
  });

  it('updateSettings invokes update_settings with settings payload', async () => {
    const next = { ...DEFAULT_SETTINGS, theme: 'Dark' as const };
    mockInvoke.mockResolvedValueOnce(next);
    const result = await updateSettings(next);
    expect(result).toEqual(next);
    expect(mockInvoke).toHaveBeenCalledWith('update_settings', { settings: next });
  });
});
