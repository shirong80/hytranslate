import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  cleanupLegacyDataDir,
  getCleanupConfirmationPhrase,
  getLegacyMigrationStatus,
  issueCleanupToken,
} from '@features/paths/ipc';

const invokeMock = vi.fn();
vi.mock('@lib/ipc/client', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

describe('paths ipc wrappers (legacy migration)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('getLegacyMigrationStatus invokes the camelCase command name', async () => {
    invokeMock.mockResolvedValue({ legacyCleanable: true, legacyDir: '/x', verified: true });
    const view = await getLegacyMigrationStatus();
    expect(invokeMock).toHaveBeenCalledWith('get_legacy_migration_status');
    expect(view.legacyCleanable).toBe(true);
  });

  it('getCleanupConfirmationPhrase returns the backend-authoritative phrase (Major 1 v3)', async () => {
    invokeMock.mockResolvedValue('HyTranslate Mac');
    const phrase = await getCleanupConfirmationPhrase();
    expect(invokeMock).toHaveBeenCalledWith('cleanup_confirmation_phrase');
    expect(phrase).toBe('HyTranslate Mac');
  });

  it('issueCleanupToken sends user-typed confirmation (Major 1 v3)', async () => {
    invokeMock.mockResolvedValue('nonce-abc');
    const token = await issueCleanupToken('HyTranslate Mac');
    expect(invokeMock).toHaveBeenCalledWith('issue_cleanup_token', {
      request: { confirmation: 'HyTranslate Mac' },
    });
    expect(token).toBe('nonce-abc');
  });

  it('cleanupLegacyDataDir sends token + confirmation (Major 1 v3)', async () => {
    invokeMock.mockResolvedValue({
      kind: 'Completed',
      backupDir: '/Users/me/legacy-backup-1',
      moved: 2,
    });
    const report = await cleanupLegacyDataDir('nonce-abc', 'HyTranslate Mac');
    expect(invokeMock).toHaveBeenCalledWith('cleanup_legacy_data_dir', {
      request: { token: 'nonce-abc', confirmation: 'HyTranslate Mac' },
    });
    expect(report.kind).toBe('Completed');
    if (report.kind === 'Completed') {
      expect(report.moved).toBe(2);
    }
  });

  it('CleanupReport variant kinds round-trip as PascalCase (Major 3)', async () => {
    invokeMock.mockResolvedValue({ kind: 'Skipped' });
    const report = await cleanupLegacyDataDir('any', 'HyTranslate Mac');
    expect(report.kind).toBe('Skipped');
  });
});
