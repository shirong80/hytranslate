import { t } from '@i18n/ko';

export type AppError =
  | { kind: 'OllamaUnavailable' }
  | { kind: 'OllamaNotRunning' }
  | { kind: 'ModelMissing'; model: string }
  | { kind: 'InputTooLong'; limit: number }
  | { kind: 'Cancelled' }
  | { kind: 'NetworkBlocked' }
  | { kind: 'Internal'; message: string };

export type AppErrorKind = AppError['kind'];

/**
 * AppError 의 사용자 노출 한국어 메시지. ko.ts 키와 변수치환 규칙을 한 곳에 모은다.
 * Phase 1 의 inline-error 내부 헬퍼를 Phase 2 에서 settings 패널 등도 재사용할 수 있게 승격.
 */
export function messageFor(error: AppError): string {
  switch (error.kind) {
    case 'InputTooLong':
      return t('errors.InputTooLong', { limit: error.limit });
    case 'ModelMissing':
      return t('errors.ModelMissing');
    case 'OllamaUnavailable':
      return t('errors.OllamaUnavailable');
    case 'OllamaNotRunning':
      return t('errors.OllamaNotRunning');
    case 'Cancelled':
      return t('errors.Cancelled');
    case 'NetworkBlocked':
      return t('errors.NetworkBlocked');
    case 'Internal':
      return t('errors.Internal');
  }
}

function hasStringField(value: object, field: string): boolean {
  return typeof (value as Record<string, unknown>)[field] === 'string';
}

function hasNumberField(value: object, field: string): boolean {
  return typeof (value as Record<string, unknown>)[field] === 'number';
}

export function isAppError(value: unknown): value is AppError {
  if (typeof value !== 'object' || value === null) return false;
  const kind = (value as { kind?: unknown }).kind;
  switch (kind) {
    case 'OllamaUnavailable':
    case 'OllamaNotRunning':
    case 'Cancelled':
    case 'NetworkBlocked':
      return true;
    case 'ModelMissing':
      return hasStringField(value, 'model');
    case 'InputTooLong':
      return hasNumberField(value, 'limit');
    case 'Internal':
      return hasStringField(value, 'message');
    default:
      return false;
  }
}
