import { test } from '@playwright/test';

// PRD §14.3 E2E — 글로벌 단축키로 floating popup 호출.
// assertion plan:
//  1) macOS 손쉬운 사용 권한이 허용된 상태에서 Cmd+Shift+T 발사.
//  2) popup 윈도우가 활성 monitor 중심에 표시되고 textarea focus.
//  3) 입력 후 debounce → 번역 결과.
//  4) Esc 로 hide → 다시 Cmd+Shift+T 발사 → textarea 가 다시 focus 받음 (POPUP_OPENED listen).
//  5) 매우 긴 입력에서 popup 높이가 monitor.height * 0.8 cap 을 넘지 않음.
test.skip('popup opens via global shortcut and translates within 5s', () => undefined);
