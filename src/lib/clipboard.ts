import { writeText as pluginWriteText } from '@tauri-apps/plugin-clipboard-manager';

/**
 * Tauri 클립보드 plugin 을 우선 사용하고, Tauri 외 환경 (vite dev / vitest jsdom) 에서는
 * navigator.clipboard 로 fallback. 호출자는 await 만 신경쓰면 된다.
 */
export async function copyText(text: string): Promise<void> {
  try {
    await pluginWriteText(text);
    return;
  } catch {
    // Tauri 환경이 아닐 수 있음 — fallback.
  }
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
  }
}
