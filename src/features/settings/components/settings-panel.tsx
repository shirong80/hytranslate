import { ArrowLeft, Check, Loader2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { t } from '@i18n/ko';
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
