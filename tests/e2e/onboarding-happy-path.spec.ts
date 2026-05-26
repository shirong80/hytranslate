import { test } from '@playwright/test';

// PRD §14.3 E2E — onboarding happy path.
// 실제 통합은 Tauri Playwright 환경 결정 후 본 follow-up 범위 외 별도 작업으로 트래킹.
// assertion plan:
//  1) welcome → environment 화면 진행 시 macOS 13+ 표시.
//  2) ollama step 에서 detect_environment 통과 후 자동 실행 backoff 가 8s 이내 success 도달.
//  3) model step 에서 추천 모델 pull 시작 → progress 이벤트 수신 → completed 도달.
//  4) permissions step 에서 시스템 설정 deep-link 노출.
//  5) history step 통과 → done.
test.skip('onboarding completes in under 60s with default recommended model', // 후속: Tauri Playwright 통합 후 skip 해제.
() => undefined);
