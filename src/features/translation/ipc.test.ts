import { beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn();
const listenMock = vi.fn();

vi.mock('@lib/ipc/client', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  listen: (...args: unknown[]) => listenMock(...args),
}));

import { attachTranslationListeners, cancelTranslation, translateStream } from './ipc';

describe('translation ipc wrappers', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it('translateStream wraps invoke with the request payload', async () => {
    invokeMock.mockResolvedValue(undefined);
    await translateStream({
      sourceText: '안녕',
      sourceLanguage: 'Korean',
      model: 'm',
      requestId: 'r1',
    });
    expect(invokeMock).toHaveBeenCalledWith('translate_stream', {
      request: { sourceText: '안녕', sourceLanguage: 'Korean', model: 'm', requestId: 'r1' },
    });
  });

  it('cancelTranslation passes requestId as the single arg', async () => {
    invokeMock.mockResolvedValue(undefined);
    await cancelTranslation('r2');
    expect(invokeMock).toHaveBeenCalledWith('cancel_translation', { requestId: 'r2' });
  });

  it('attachTranslationListeners only registers provided callbacks and returns aggregate unlisten', async () => {
    const offFns = [vi.fn(), vi.fn()];
    let callIdx = 0;
    listenMock.mockImplementation(async () => offFns[callIdx++]);

    const onChunk = vi.fn();
    const onError = vi.fn();
    const unlisten = await attachTranslationListeners({ onChunk, onError });

    expect(listenMock).toHaveBeenCalledTimes(2);
    expect(listenMock.mock.calls[0]?.[0]).toBe('translation:chunk');
    expect(listenMock.mock.calls[1]?.[0]).toBe('translation:error');
    unlisten();
    for (const off of offFns) {
      expect(off).toHaveBeenCalledTimes(1);
    }
  });
});
