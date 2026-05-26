/* eslint-disable import-x/order -- vi.mock 호출이 import 보다 먼저 hoist 되어야 하므로 vitest 를 먼저 import. */
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(),
  readImage: vi.fn(),
}));

import { readImage, readText } from '@tauri-apps/plugin-clipboard-manager';

import { readClipboard } from './ipc';
/* eslint-enable import-x/order */

const readTextMock = readText as unknown as ReturnType<typeof vi.fn>;
const readImageMock = readImage as unknown as ReturnType<typeof vi.fn>;

describe('readClipboard', () => {
  beforeEach(() => {
    readTextMock.mockReset();
    readImageMock.mockReset();
  });

  it('returns text when readText resolves with non-empty string', async () => {
    readTextMock.mockResolvedValueOnce('안녕하세요');
    const result = await readClipboard();
    expect(result).toEqual({ kind: 'text', text: '안녕하세요' });
    expect(readImageMock).not.toHaveBeenCalled();
  });

  it('returns unsupported when text is empty and image probe succeeds', async () => {
    readTextMock.mockResolvedValueOnce('');
    readImageMock.mockResolvedValueOnce({});
    const result = await readClipboard();
    expect(result).toEqual({ kind: 'unsupported' });
  });

  it('returns empty when text is empty and image probe throws', async () => {
    readTextMock.mockResolvedValueOnce('');
    readImageMock.mockRejectedValueOnce(new Error('no image'));
    const result = await readClipboard();
    expect(result).toEqual({ kind: 'empty' });
  });

  it('returns readFailed when readText itself throws (Major 3 2차 보강)', async () => {
    readTextMock.mockRejectedValueOnce(new Error('permission denied'));
    const result = await readClipboard();
    expect(result.kind).toBe('readFailed');
    if (result.kind === 'readFailed') {
      expect(result.error.kind).toBe('ClipboardReadFailed');
      if (result.error.kind === 'ClipboardReadFailed') {
        expect(result.error.message).toBe('permission denied');
      }
    }
    expect(readImageMock).not.toHaveBeenCalled();
  });
});
