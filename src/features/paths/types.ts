export interface MigrationStatusView {
  legacyCleanable: boolean;
  legacyDir: string | null;
  verified: boolean;
}

export type CleanupReport =
  | { kind: 'Skipped' }
  | { kind: 'Completed'; backupDir: string; moved: number };
