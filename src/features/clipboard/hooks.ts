import { useCallback } from 'react';

import { t } from '@i18n/ko';
import { messageFor } from '@lib/ipc/errors';

import { readClipboard } from './ipc';

interface UsePasteFromClipboardOptions {
  onText: (text: string) => void;
  /**
   * empty / unsupported / readFailed 각각에 다른 메시지를 전달한다. 호출자는
   * 메시지를 inline UI 에 표시하고 1~3초 후 자동 소멸시키거나 다시 시도 버튼을 노출.
   */
  onError?: (message: string) => void;
}

export function usePasteFromClipboard({ onText, onError }: UsePasteFromClipboardOptions) {
  return useCallback(async () => {
    const result = await readClipboard();
    switch (result.kind) {
      case 'text':
        onText(result.text);
        return;
      case 'empty':
        onError?.(t('errors.ClipboardEmpty'));
        return;
      case 'unsupported':
        onError?.(t('errors.ClipboardUnsupported'));
        return;
      case 'readFailed':
        onError?.(messageFor(result.error));
        return;
    }
  }, [onText, onError]);
}
