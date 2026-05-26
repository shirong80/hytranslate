import { test } from '@playwright/test';

// PRD §14.3 E2E — main window 번역 흐름.
// assertion plan:
//  1) main window 가 열린 상태에서 textarea 에 한국어 입력.
//  2) 500ms debounce 후 typing → detecting → translating → completed 상태 전이.
//  3) Auto 입력의 경우 backend resolvedLanguage badge 가 "자동: 한국어" 로 노출.
//  4) 결과 영역에 streaming chunk 가 도착하고, completed 시점에 fullText 가 표시.
//  5) Cmd+Enter 로 재번역 시 동일 흐름 반복.
test.skip('main window translates Korean input end-to-end with debounce', () => undefined);
