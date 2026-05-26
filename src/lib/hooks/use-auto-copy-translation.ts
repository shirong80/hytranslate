import { useEffect, useRef } from 'react';

import { useTranslationStore } from '@features/translation/store';

import { copyText } from '../clipboard';

/**
 * `enabled` true 일 때, 번역이 `completed` 상태로 진입하면 결과를 클립보드에 자동 복사.
 *
 * - 같은 requestId 에 대해서는 중복 복사하지 않는다 (cancel/retry 사이클에서도 안정).
 * - cancel/error 상태에서는 복사하지 않는다.
 * - lib 영역의 cross-cutting 훅이므로 translation feature store 를 import 해도 규칙
 *   위반이 아니다. 다른 feature store 를 합치는 cross-feature 의존성은 만들지 않는다.
 */
export function useAutoCopyTranslation(enabled: boolean): void {
  const status = useTranslationStore((s) => s.status);
  const output = useTranslationStore((s) => s.output);
  const requestId = useTranslationStore((s) => s.requestId);
  const lastCopiedReqRef = useRef<string | null>(null);

  useEffect(() => {
    if (!enabled) return;
    if (status !== 'completed') return;
    if (!output) return;
    // requestId 가 completed 시점에 null 로 비워질 수 있어 output 으로 dedup 보강.
    const dedupKey = requestId ?? output;
    if (lastCopiedReqRef.current === dedupKey) return;
    lastCopiedReqRef.current = dedupKey;
    void copyText(output).catch(() => {
      // 실패는 UX 상 무시 — 사용자가 수동 복사 버튼으로 복구.
    });
  }, [enabled, status, output, requestId]);
}
