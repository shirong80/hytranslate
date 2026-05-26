export type AppError =
  | { kind: 'OllamaUnavailable' }
  | { kind: 'OllamaNotRunning' }
  | { kind: 'ModelMissing'; model: string }
  | { kind: 'InputTooLong'; limit: number }
  | { kind: 'Cancelled' }
  | { kind: 'NetworkBlocked' }
  | { kind: 'Internal'; message: string };

export type AppErrorKind = AppError['kind'];

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
