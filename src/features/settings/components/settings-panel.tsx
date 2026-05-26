import { AlertCircle, ArrowLeft, Check, ExternalLink, Loader2, Trash2 } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { useHistoryStore } from '@features/history/store';
import {
  cleanupLegacyDataDir,
  getCleanupConfirmationPhrase,
  getLegacyMigrationStatus,
  issueCleanupToken,
} from '@features/paths/ipc';
import type { MigrationStatusView } from '@features/paths/types';
import { t } from '@i18n/ko';
import { invoke } from '@lib/ipc/client';
import { messageFor } from '@lib/ipc/errors';

import { useSettingsStore } from '../store';
import { type Settings, type Theme } from '../types';

const HY_MT2_7B = 'hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M';
const HY_MT2_1_8B = 'hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M';

interface SettingsPanelProps {
  onBack: () => void;
}

export function SettingsPanel({ onBack }: SettingsPanelProps) {
  const settings = useSettingsStore((s) => s.settings);
  const saving = useSettingsStore((s) => s.saving);
  const error = useSettingsStore((s) => s.error);
  const save = useSettingsStore((s) => s.save);

  const [draft, setDraft] = useState<Settings>(settings);
  const [savedFlash, setSavedFlash] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEffect(() => {
    if (!savedFlash) return;
    const timer = window.setTimeout(() => setSavedFlash(false), 1500);
    return () => window.clearTimeout(timer);
  }, [savedFlash]);

  const dirty = JSON.stringify(draft) !== JSON.stringify(settings);

  const handleSave = async () => {
    await save(draft);
    if (!useSettingsStore.getState().error) setSavedFlash(true);
  };

  return (
    <div className="flex h-full flex-col gap-4 p-6">
      <header className="flex items-center gap-3">
        <button
          type="button"
          onClick={onBack}
          aria-label={t('nav.back')}
          className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
        >
          <ArrowLeft className="size-3.5" aria-hidden />
          {t('nav.back')}
        </button>
        <h1 className="text-lg font-medium tracking-tight text-neutral-900 dark:text-neutral-100">
          {t('settings.title')}
        </h1>
      </header>

      <div className="flex flex-1 flex-col gap-6 overflow-auto">
        <Section title={t('settings.section.translation')}>
          <Field label={t('settings.activeModel.label')} htmlFor="active-model">
            <select
              id="active-model"
              value={draft.activeModel}
              onChange={(e) => setDraft({ ...draft, activeModel: e.target.value })}
              className="w-full rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm text-neutral-900 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100"
            >
              <option value={HY_MT2_7B}>{t('settings.activeModel.hy7b')}</option>
              <option value={HY_MT2_1_8B}>{t('settings.activeModel.hy1_8b')}</option>
            </select>
          </Field>

          <Field label={t('settings.ollamaEndpoint.label')} htmlFor="ollama-endpoint">
            <input
              id="ollama-endpoint"
              type="text"
              value={draft.ollamaEndpoint}
              onChange={(e) => setDraft({ ...draft, ollamaEndpoint: e.target.value })}
              spellCheck={false}
              autoComplete="off"
              className="w-full rounded-md border border-neutral-300 bg-white px-3 py-1.5 font-mono text-xs text-neutral-900 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100"
            />
            <p className="text-xs text-neutral-500 dark:text-neutral-500">
              {t('settings.ollamaEndpoint.help')}
            </p>
          </Field>

          <Toggle
            id="auto-copy"
            label={t('settings.autoCopy.label')}
            checked={draft.autoCopyAfterTranslation}
            onChange={(v) => setDraft({ ...draft, autoCopyAfterTranslation: v })}
          />
          <Toggle
            id="save-history"
            label={t('settings.saveHistory.label')}
            checked={draft.saveHistory}
            onChange={(v) => setDraft({ ...draft, saveHistory: v })}
          />
          <DeleteAllHistoryRow />
        </Section>

        <Section title={t('settings.section.shortcut')}>
          <Field label={t('settings.globalHotkey.label')} htmlFor="global-hotkey">
            <input
              id="global-hotkey"
              type="text"
              value={draft.globalHotkey}
              onChange={(e) => setDraft({ ...draft, globalHotkey: e.target.value })}
              spellCheck={false}
              autoComplete="off"
              placeholder="Cmd+Shift+T"
              className="w-full rounded-md border border-neutral-300 bg-white px-3 py-1.5 font-mono text-xs text-neutral-900 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100"
            />
            <p className="text-xs text-neutral-500 dark:text-neutral-500">
              {t('settings.globalHotkey.help')}
            </p>
            <button
              type="button"
              onClick={() => {
                invoke<void>('open_accessibility_settings').catch(() => undefined);
              }}
              className="mt-1 inline-flex w-fit items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-2 py-1 text-[11px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
            >
              <ExternalLink className="size-3" aria-hidden />
              {t('errors.action.openSystemSettings')}
            </button>
          </Field>
        </Section>

        <Section title={t('settings.section.system')}>
          <Toggle
            id="start-at-login"
            label={t('settings.startAtLogin.label')}
            checked={draft.startAtLogin}
            onChange={(v) => setDraft({ ...draft, startAtLogin: v })}
          />
          <Toggle
            id="hide-dock-icon"
            label={t('settings.hideDockIcon.label')}
            checked={draft.hideDockIcon}
            onChange={(v) => setDraft({ ...draft, hideDockIcon: v })}
          />
        </Section>

        <Section title={t('settings.section.appearance')}>
          <Field label={t('settings.theme.label')} htmlFor="theme">
            <select
              id="theme"
              value={draft.theme}
              onChange={(e) => setDraft({ ...draft, theme: e.target.value as Theme })}
              className="w-full rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm text-neutral-900 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100"
            >
              <option value="System">{t('settings.theme.system')}</option>
              <option value="Light">{t('settings.theme.light')}</option>
              <option value="Dark">{t('settings.theme.dark')}</option>
            </select>
          </Field>
        </Section>

        <Section title={t('settings.section.data')}>
          <LegacyMigrationBanner />
        </Section>
      </div>

      <footer className="flex items-center justify-between border-t border-neutral-200 pt-4 dark:border-neutral-800">
        {error ? (
          <span className="text-xs text-rose-600 dark:text-rose-400">{messageFor(error)}</span>
        ) : (
          <span className="text-xs text-neutral-500 dark:text-neutral-500" />
        )}
        <button
          type="button"
          onClick={handleSave}
          disabled={!dirty || saving}
          className="inline-flex items-center gap-2 rounded-md border border-neutral-900 bg-neutral-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-100 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
        >
          {saving ? <Loader2 className="size-4 animate-spin" aria-hidden /> : null}
          {savedFlash ? <Check className="size-4" aria-hidden /> : null}
          {saving
            ? t('settings.action.saving')
            : savedFlash
              ? t('settings.action.saved')
              : t('settings.action.save')}
        </button>
      </footer>
    </div>
  );
}

function LegacyMigrationBanner() {
  const [status, setStatus] = useState<MigrationStatusView | null>(null);
  const [confirmationPhrase, setConfirmationPhrase] = useState<string | null>(null);
  const [typedConfirmation, setTypedConfirmation] = useState('');
  const [busy, setBusy] = useState(false);
  const [completedMessage, setCompletedMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [view, phrase] = await Promise.all([
        getLegacyMigrationStatus(),
        getCleanupConfirmationPhrase().catch(() => null),
      ]);
      setStatus(view);
      setConfirmationPhrase(phrase);
    } catch {
      // 자동 단계 outcome 이 등록되지 않은 환경(개발 등)에서는 silent.
      setStatus(null);
      setConfirmationPhrase(null);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleCleanup = useCallback(async () => {
    if (!confirmationPhrase) return;
    const typed = typedConfirmation.trim();
    if (typed !== confirmationPhrase) {
      setErrorMessage(
        t('settings.legacyMigration.confirmPhraseMismatch', { phrase: confirmationPhrase }),
      );
      return;
    }
    if (!window.confirm(t('settings.legacyMigration.confirm'))) return;
    setBusy(true);
    setErrorMessage(null);
    try {
      // code-review v1 follow-up review §10 (Major 1 v3) — backend 가 user-typed
      // confirmation phrase 를 검증해야만 token 을 발급한다. cleanup 시점에도 같은
      // confirmation 을 다시 검증 (defense in depth). renderer 가 token/cleanup
      // 명령만으로 cleanup 을 우회 실행할 수 없다.
      const token = await issueCleanupToken(typed);
      const report = await cleanupLegacyDataDir(token, typed);
      if (report.kind === 'Completed') {
        setCompletedMessage(
          t('settings.legacyMigration.completed', {
            backupDir: report.backupDir,
            moved: report.moved,
          }),
        );
      }
      setTypedConfirmation('');
      await refresh();
    } catch (e) {
      setErrorMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [confirmationPhrase, typedConfirmation, refresh]);

  if (!status || !status.legacyCleanable || !confirmationPhrase) {
    if (completedMessage) {
      return <p className="text-xs text-neutral-600 dark:text-neutral-400">{completedMessage}</p>;
    }
    return null;
  }

  const typedTrimmed = typedConfirmation.trim();
  const phraseMatches = typedTrimmed === confirmationPhrase;

  return (
    <div className="flex flex-col gap-2 rounded-md border border-amber-300 bg-amber-50 p-3 text-xs text-amber-900 dark:border-amber-700 dark:bg-amber-950/40 dark:text-amber-100">
      <div className="flex items-start gap-2">
        <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
        <p>
          {t('settings.legacyMigration.banner', {
            legacyDir: status.legacyDir ?? '',
          })}
        </p>
      </div>
      <label htmlFor="legacy-cleanup-confirm" className="flex flex-col gap-1">
        <span className="text-[11px]">
          {t('settings.legacyMigration.confirmPhraseLabel', { phrase: confirmationPhrase })}
        </span>
        <input
          id="legacy-cleanup-confirm"
          type="text"
          value={typedConfirmation}
          onChange={(e) => setTypedConfirmation(e.target.value)}
          placeholder={t('settings.legacyMigration.confirmPhrasePlaceholder')}
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
          className="w-full rounded-md border border-amber-400 bg-white px-2 py-1 font-mono text-[11px] text-neutral-900 focus:border-amber-500 focus:outline-none focus:ring-1 focus:ring-amber-500 dark:border-amber-700 dark:bg-neutral-900 dark:text-neutral-100"
        />
      </label>
      {errorMessage ? (
        <p className="text-[11px] text-rose-600 dark:text-rose-400">{errorMessage}</p>
      ) : null}
      <div>
        <button
          type="button"
          onClick={handleCleanup}
          disabled={busy || !phraseMatches}
          className="inline-flex items-center gap-1.5 rounded-md border border-amber-400 bg-white px-2 py-1 text-[11px] font-medium text-amber-900 hover:border-amber-500 hover:bg-amber-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-amber-700 dark:bg-neutral-900 dark:text-amber-100 dark:hover:bg-neutral-800"
        >
          {busy ? <Loader2 className="size-3 animate-spin" aria-hidden /> : null}
          {busy ? t('settings.legacyMigration.cleanupBusy') : t('settings.legacyMigration.cleanup')}
        </button>
      </div>
    </div>
  );
}

function DeleteAllHistoryRow() {
  const removeAll = useHistoryStore((s) => s.removeAll);
  const [busy, setBusy] = useState(false);
  const handleClick = async () => {
    if (!window.confirm(t('history.deleteAll.confirm'))) return;
    setBusy(true);
    try {
      await removeAll();
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex items-center justify-between gap-3 text-xs text-neutral-700 dark:text-neutral-300">
      <span>{t('settings.deleteAllHistory.label')}</span>
      <button
        type="button"
        onClick={handleClick}
        disabled={busy}
        className="inline-flex items-center gap-1 rounded-md border border-rose-200 bg-white px-2 py-1 text-xs text-rose-700 hover:border-rose-300 hover:bg-rose-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-rose-900 dark:bg-neutral-900 dark:text-rose-300 dark:hover:bg-rose-950"
      >
        <Trash2 className="size-3" aria-hidden />
        {t('history.deleteAll')}
      </button>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xs font-medium uppercase tracking-wider text-neutral-500 dark:text-neutral-500">
        {title}
      </h2>
      <div className="flex flex-col gap-4 rounded-lg border border-neutral-200 bg-neutral-50 p-4 dark:border-neutral-800 dark:bg-neutral-900/40">
        {children}
      </div>
    </section>
  );
}

function Field({
  label,
  htmlFor,
  children,
}: {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <label htmlFor={htmlFor} className="text-xs text-neutral-700 dark:text-neutral-300">
        {label}
      </label>
      {children}
    </div>
  );
}

function Toggle({
  id,
  label,
  checked,
  onChange,
}: {
  id: string;
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label
      htmlFor={id}
      className="flex cursor-pointer items-center justify-between gap-3 text-xs text-neutral-700 dark:text-neutral-300"
    >
      <span>{label}</span>
      <input
        id={id}
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="size-4 cursor-pointer accent-brand"
      />
    </label>
  );
}
