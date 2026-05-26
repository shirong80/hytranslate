import { describe, expect, it } from 'vitest';

import { isAppError } from './errors';

describe('isAppError', () => {
  it('accepts payload-less variants', () => {
    expect(isAppError({ kind: 'OllamaUnavailable' })).toBe(true);
    expect(isAppError({ kind: 'OllamaNotRunning' })).toBe(true);
    expect(isAppError({ kind: 'Cancelled' })).toBe(true);
    expect(isAppError({ kind: 'NetworkBlocked' })).toBe(true);
  });

  it('requires model:string for ModelMissing', () => {
    expect(isAppError({ kind: 'ModelMissing', model: 'hf.co/x' })).toBe(true);
    expect(isAppError({ kind: 'ModelMissing' })).toBe(false);
    expect(isAppError({ kind: 'ModelMissing', model: 42 })).toBe(false);
  });

  it('requires limit:number for InputTooLong', () => {
    expect(isAppError({ kind: 'InputTooLong', limit: 30000 })).toBe(true);
    expect(isAppError({ kind: 'InputTooLong' })).toBe(false);
    expect(isAppError({ kind: 'InputTooLong', limit: '30000' })).toBe(false);
  });

  it('requires message:string for Internal', () => {
    expect(isAppError({ kind: 'Internal', message: 'boom' })).toBe(true);
    expect(isAppError({ kind: 'Internal' })).toBe(false);
    expect(isAppError({ kind: 'Internal', message: null })).toBe(false);
  });

  it('rejects unknown kinds and non-objects', () => {
    expect(isAppError({ kind: 'Bogus' })).toBe(false);
    expect(isAppError(null)).toBe(false);
    expect(isAppError(undefined)).toBe(false);
    expect(isAppError('OllamaUnavailable')).toBe(false);
    expect(isAppError({})).toBe(false);
  });
});
