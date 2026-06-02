import { invoke } from '@lib/ipc/client';

// popup window 의 Rust command 계약을 typed wrapper 로 고정한다 — entrypoint 에 magic string 을
// 흩뿌리지 않는다(architecture: 컴포넌트는 invoke() 직접 호출 금지).
export function resizePopup(height: number): Promise<void> {
  return invoke<void>('resize_popup', { height });
}

export function hidePopup(): Promise<void> {
  return invoke<void>('hide_popup');
}
