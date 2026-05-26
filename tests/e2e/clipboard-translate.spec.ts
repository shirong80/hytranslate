import { test } from '@playwright/test';

// PRD §14.3 E2E — 클립보드 번역.
// assertion plan:
//  1) 시스템 클립보드에 한국어 텍스트 주입.
//  2) 메뉴바 popover 열어 "클립보드 번역" 클릭.
//  3) source textarea 가 채워지고 자동 debounce 번역 시작.
//  4) 빈 클립보드 / 이미지 클립보드에서는 각각 다른 inline 안내가 노출 (empty / unsupported).
//  5) clipboard-manager allow-read-text 미허용 시 readFailed 분기로 권한 안내.
test.skip('clipboard paste routes text/empty/unsupported/readFailed into distinct UI', () =>
  undefined);
