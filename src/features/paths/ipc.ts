import { invoke } from '@lib/ipc/client';

import type { CleanupReport, MigrationStatusView } from './types';

export async function getLegacyMigrationStatus(): Promise<MigrationStatusView> {
  return invoke<MigrationStatusView>('get_legacy_migration_status');
}

/**
 * UI 가 사용자에게 보여줄 confirmation phrase (legacy 폴더 이름). backend 가 권위적인 출처.
 * renderer 가 임의 phrase 를 주입할 수 없게 함. 반환값이 null 이면 cleanup CTA 비노출.
 * code-review v1 follow-up review §10 (Major 1 v3).
 */
export async function getCleanupConfirmationPhrase(): Promise<string | null> {
  return invoke<string | null>('cleanup_confirmation_phrase');
}

/**
 * 사용자가 입력박스에 backend phrase 를 직접 타이핑한 직후 호출. backend 가 phrase
 * 일치를 검증해야만 1회용 토큰을 발급한다. confirm 모달 + UI input 두 단계 user-intent
 * 신호가 backend 검증을 통과해야 cleanup 이 가능 (Major 1 v3).
 */
export async function issueCleanupToken(confirmation: string): Promise<string> {
  return invoke<string>('issue_cleanup_token', {
    request: { confirmation },
  });
}

/** token + confirmation 둘 다 backend 가 재검증 (defense in depth). */
export async function cleanupLegacyDataDir(
  token: string,
  confirmation: string,
): Promise<CleanupReport> {
  return invoke<CleanupReport>('cleanup_legacy_data_dir', {
    request: { token, confirmation },
  });
}
