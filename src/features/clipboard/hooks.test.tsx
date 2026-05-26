/* eslint-disable import-x/order -- vi.mock hoisting */
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(),
  readImage: vi.fn(),
}));

import { readImage, readText } from '@tauri-apps/plugin-clipboard-manager';

import { usePasteFromClipboard } from './hooks';
/* eslint-enable import-x/order */

const readTextMock = readText as unknown as ReturnType<typeof vi.fn>;
const readImageMock = readImage as unknown as ReturnType<typeof vi.fn>;

describe('usePasteFromClipboard', () => {
  beforeEach(() => {
    readTextMock.mockReset();
    readImageMock.mockReset();
  });

  it('routes text result to onText', async () => {
    readTextMock.mockResolvedValue('hello');
    const onText = vi.fn();
    const onError = vi.fn();
    const { result } = renderHook(() => usePasteFromClipboard({ onText, onError }));
    await act(async () => {
      await result.current();
    });
    expect(onText).toHaveBeenCalledWith('hello');
    expect(onError).not.toHaveBeenCalled();
  });

  it('routes empty clipboard to onError with ClipboardEmpty message', async () => {
    readTextMock.mockResolvedValue('');
    readImageMock.mockRejectedValue(new Error('no image'));
    const onText = vi.fn();
    const onError = vi.fn();
    const { result } = renderHook(() => usePasteFromClipboard({ onText, onError }));
    await act(async () => {
      await result.current();
    });
    expect(onText).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError.mock.calls[0]?.[0]).toContain('클립보드에 텍스트');
  });

  it('routes image clipboard to onError with ClipboardUnsupported message', async () => {
    readTextMock.mockResolvedValue('');
    readImageMock.mockResolvedValue({});
    const onText = vi.fn();
    const onError = vi.fn();
    const { result } = renderHook(() => usePasteFromClipboard({ onText, onError }));
    await act(async () => {
      await result.current();
    });
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError.mock.calls[0]?.[0]).toContain('이미지');
  });

  it('routes readText throw to onError with ClipboardReadFailed message', async () => {
    readTextMock.mockRejectedValue(new Error('permission denied'));
    const onText = vi.fn();
    const onError = vi.fn();
    const { result } = renderHook(() => usePasteFromClipboard({ onText, onError }));
    await act(async () => {
      await result.current();
    });
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError.mock.calls[0]?.[0]).toContain('손쉬운 사용');
  });
});
