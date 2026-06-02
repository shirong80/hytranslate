import { beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@lib/ipc/client', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { hidePopup, resizePopup } from './ipc';

describe('popup ipc wrappers', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('resizePopup wraps invoke with the height payload', async () => {
    invokeMock.mockResolvedValue(undefined);
    await resizePopup(528);
    expect(invokeMock).toHaveBeenCalledWith('resize_popup', { height: 528 });
  });

  it('hidePopup invokes hide_popup with no payload', async () => {
    invokeMock.mockResolvedValue(undefined);
    await hidePopup();
    expect(invokeMock).toHaveBeenCalledWith('hide_popup');
  });
});
