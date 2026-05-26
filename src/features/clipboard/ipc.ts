import { readImage, readText } from '@tauri-apps/plugin-clipboard-manager';

import type { ClipboardReadResult } from './types';

/**
 * macOS 클립보드 감지 — Tauri 2 plugin-clipboard-manager 는 `readText` / `readImage`
 * 만 노출하고 Finder 파일(NSFilenamesPboardType)을 별도로 읽지 못한다.
 *
 * 절차:
 *  1) readText 성공 + non-empty → text
 *  2) readText 성공 + empty     → readImage 시도
 *      - 성공 → unsupported (이미지)
 *      - throw → empty (이미지 없음 정상; file/empty 케이스 포함)
 *  3) readText throw            → readFailed (권한 / 플러그인 통합 문제)
 */
export async function readClipboard(): Promise<ClipboardReadResult> {
  let text: string;
  try {
    text = await readText();
  } catch (err) {
    return {
      kind: 'readFailed',
      error: {
        kind: 'ClipboardReadFailed',
        message: err instanceof Error ? err.message : String(err),
      },
    };
  }
  if (text && text.length > 0) {
    return { kind: 'text', text };
  }
  try {
    await readImage();
    return { kind: 'unsupported' };
  } catch {
    return { kind: 'empty' };
  }
}
