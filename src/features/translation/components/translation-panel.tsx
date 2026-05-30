import { Check, Copy, History, Loader2, RefreshCcw, Settings as SettingsIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { t } from '@i18n/ko';
import { invoke } from '@lib/ipc/client';

import { useTranslationStore } from '../store';
import { MAIN_INPUT_LIMIT } from '../types';
import { useTranslationController } from '../use-translation-controller';

import { InlineError } from './inline-error';
import { SourceLanguageSelect } from './source-language-select';

interface TranslationPanelProps {
  onOpenSettings?: () => void;
  onOpenHistory?: () => void;
}

export function TranslationPanel({ onOpenSettings, onOpenHistory }: TranslationPanelProps = {}) {
  const sourceText = useTranslationStore((s) => s.sourceText);
  const sourceLanguage = useTranslationStore((s) => s.sourceLanguage);
  const resolvedLanguage = useTranslationStore((s) => s.resolvedLanguage);
  const output = useTranslationStore((s) => s.output);
  const status = useTranslationStore((s) => s.status);
  const error = useTranslationStore((s) => s.error);
  const durationMs = useTranslationStore((s) => s.durationMs);
  const model = useTranslationStore((s) => s.model);

  const setSourceText = useTranslationStore((s) => s.setSourceText);
  const setSourceLanguage = useTranslationStore((s) => s.setSourceLanguage);
  const copyError = useTranslationStore((s) => s.copyError);
  const setCopyError = useTranslationStore((s) => s.setCopyError);

  const { runImmediately, saveAndClear } = useTranslationController();
  const [copied, setCopied] = useState(false);

  const charCount = [...sourceText].length;
  const overLimit = charCount > MAIN_INPUT_LIMIT;

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        void saveAndClear();
      }
    },
    [saveAndClear],
  );

  const handleCopy = useCallback(async () => {
    if (!output) return;
    try {
      await navigator.clipboard.writeText(output);
      setCopied(true);
      setCopyError(null);
    } catch (err) {
      setCopyError({
        kind: 'CopyFailed',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, [output, setCopyError]);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(timer);
  }, [copied]);

  useEffect(() => {
    if (!copyError) return;
    const timer = window.setTimeout(() => setCopyError(null), 1500);
    return () => window.clearTimeout(timer);
  }, [copyError, setCopyError]);

  const handleOpenOllamaDownload = useCallback(() => {
    invoke<void>('open_ollama_download_page').catch(() => {
      // 침묵 — 백엔드 spawn 실패는 매우 드물고 inline 에러로 추가 노출하지 않음
    });
  }, []);

  return (
    <div className="flex h-full flex-col gap-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-lg font-medium tracking-tight text-neutral-900 dark:text-neutral-100">
          {t('app.title')}
        </h1>
        <div className="flex items-center gap-2">
          <ModelBadge model={model} status={status} durationMs={durationMs} />
          {onOpenHistory ? (
            <button
              type="button"
              onClick={onOpenHistory}
              aria-label={t('nav.history')}
              className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
            >
              <History className="size-3.5" aria-hidden />
              {t('nav.history')}
            </button>
          ) : null}
          {onOpenSettings ? (
            <button
              type="button"
              onClick={onOpenSettings}
              aria-label={t('nav.settings')}
              className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
            >
              <SettingsIcon className="size-3.5" aria-hidden />
              {t('nav.settings')}
            </button>
          ) : null}
        </div>
      </header>

      <div className="grid flex-1 grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <SourceLanguageSelect
              value={sourceLanguage}
              onChange={setSourceLanguage}
              resolvedLanguage={resolvedLanguage}
            />
            <CharCount count={charCount} limit={MAIN_INPUT_LIMIT} overLimit={overLimit} />
          </div>
          <textarea
            value={sourceText}
            onChange={(event) => setSourceText(event.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('translation.input.placeholder')}
            spellCheck={false}
            className="flex-1 resize-none rounded-md border border-neutral-300 bg-white p-3 font-sans text-sm leading-relaxed text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-600"
          />
        </section>

        <section className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <StatusIndicator status={status} />
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={runImmediately}
                disabled={sourceText.trim().length === 0}
                className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
              >
                <RefreshCcw className="size-3.5" aria-hidden />
                {t('translation.output.retranslate')}
              </button>
              <button
                type="button"
                onClick={handleCopy}
                disabled={!output}
                className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
              >
                {copied ? (
                  <>
                    <Check className="size-3.5" aria-hidden />
                    {t('translation.output.copied')}
                  </>
                ) : (
                  <>
                    <Copy className="size-3.5" aria-hidden />
                    {t('translation.output.copy')}
                  </>
                )}
              </button>
            </div>
          </div>
          {copyError ? (
            <div
              role="alert"
              className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-xs text-amber-900 dark:border-amber-700 dark:bg-amber-950/40 dark:text-amber-100"
            >
              {t('errors.CopyFailed')}
            </div>
          ) : null}
          {error ? (
            <InlineError
              error={error}
              onRetry={runImmediately}
              onOpenOllamaDownload={handleOpenOllamaDownload}
            />
          ) : (
            <div
              role="status"
              aria-live="polite"
              className="flex-1 overflow-auto whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50 p-3 font-sans text-sm leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
            >
              {output ? (
                output
              ) : (
                <span className="text-neutral-400 dark:text-neutral-600">
                  {t('translation.output.placeholder')}
                </span>
              )}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

function CharCount({
  count,
  limit,
  overLimit,
}: {
  count: number;
  limit: number;
  overLimit: boolean;
}) {
  return (
    <span
      className={
        overLimit
          ? 'text-xs font-medium text-rose-600 dark:text-rose-400'
          : 'text-xs text-neutral-500 dark:text-neutral-500'
      }
    >
      {t('translation.input.charCount', { count, limit })}
    </span>
  );
}

function StatusIndicator({
  status,
}: {
  status: ReturnType<typeof useTranslationStore.getState>['status'];
}) {
  if (status === 'translating') {
    return (
      <span
        role="status"
        className="inline-flex items-center gap-1.5 text-xs text-neutral-500 dark:text-neutral-400"
      >
        <Loader2 className="size-3.5 animate-spin" aria-hidden />
        {t('translation.status.translating')}
      </span>
    );
  }
  return null;
}

function ModelBadge({
  model,
  status,
  durationMs,
}: {
  model: string;
  status: ReturnType<typeof useTranslationStore.getState>['status'];
  durationMs: number | null;
}) {
  const showDuration = status === 'completed' && durationMs != null;
  return (
    <div className="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400">
      <span className="font-mono">{shortModelName(model)}</span>
      {showDuration && (
        <span className="rounded-md bg-neutral-100 px-1.5 py-0.5 font-mono text-[10px] text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
          {t('translation.status.duration', { ms: durationMs })}
        </span>
      )}
    </div>
  );
}

function shortModelName(full: string): string {
  if (full.includes('7B')) return 'Hy-MT2 7B';
  if (full.includes('1.8B') || full.includes('1_8B')) return 'Hy-MT2 1.8B';
  return full.split(':')[0] ?? full;
}
