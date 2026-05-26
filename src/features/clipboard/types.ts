import type { AppError } from '@lib/ipc/errors';

/**
 * 클립보드 읽기 결과의 4가지 분기. PRD §6.4: text/image/file/empty 별 다른 안내.
 *
 * `readText()` 의 throw 는 `empty` 로 가리지 않고 `readFailed` 로 분리한다.
 * 권한 / 플러그인 통합 문제를 "텍스트 없음" 으로 숨기지 않기 위해서 (코드리뷰 Major 3 2차 반영).
 */
export type ClipboardReadResult =
  | { kind: 'text'; text: string }
  | { kind: 'empty' }
  | { kind: 'unsupported' }
  | { kind: 'readFailed'; error: AppError };
