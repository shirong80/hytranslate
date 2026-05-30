import { currentMonitor, getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { Check, ClipboardPaste, Copy, Loader2, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';

import { usePasteFromClipboard } from '@features/clipboard/hooks';
import { useSettingsStore } from '@features/settings/store';
import { SourceLanguageSelect } from '@features/translation/components/source-language-select';
import { useTranslationStore } from '@features/translation/store';
import { POPUP_INPUT_LIMIT } from '@features/translation/types';
import { useTranslationController } from '@features/translation/use-translation-controller';
import { t } from '@i18n/ko';
import { copyText } from '@lib/clipboard';
import { useAutoCopyTranslation } from '@lib/hooks/use-auto-copy-translation';
import { invoke, listen } from '@lib/ipc/client';
import { messageFor } from '@lib/ipc/errors';
import { POPUP_OPENED } from '@lib/ipc/events';
import { applyTheme, type ThemeMode } from '@lib/theme';

import '@styles/globals.css';

function PopupApp() {
  const load = useSettingsStore((s) => s.load);
  const loaded = useSettingsStore((s) => s.loaded);
  const theme = useSettingsStore((s) => s.settings.theme);
  const activeModel = useSettingsStore((s) => s.settings.activeModel);
  const autoCopy = useSettingsStore((s) => s.settings.autoCopyAfterTranslation);

  const sourceText = useTranslationStore((s) => s.sourceText);
  const sourceLanguage = useTranslationStore((s) => s.sourceLanguage);
  const resolvedLanguage = useTranslationStore((s) => s.resolvedLanguage);
  const output = useTranslationStore((s) => s.output);
  const status = useTranslationStore((s) => s.status);
  const error = useTranslationStore((s) => s.error);

  const setSourceText = useTranslationStore((s) => s.setSourceText);
  const setSourceLanguage = useTranslationStore((s) => s.setSourceLanguage);
  const setModel = useTranslationStore((s) => s.setModel);

  const { saveAndClear } = useTranslationController({ inputLimit: POPUP_INPUT_LIMIT });
  useAutoCopyTranslation(autoCopy);

  const [copied, setCopied] = useState(false);
  const [pasteError, setPasteError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // 글로벌 단축키로 윈도우가 열릴 때 자동 포커스. autoFocus prop 대신 ref 로
    // 명시 — a11y 린트가 autoFocus 를 금지하지만 popup 의 UX 상 즉시 포커스 필수.
    textareaRef.current?.focus();
  }, []);

  // Major 4 — 단축키로 다시 열릴 때마다 textarea focus 보장.
  useEffect(() => {
    let off: (() => void) | undefined;
    let cancelled = false;
    listen<void>(POPUP_OPENED, () => {
      textareaRef.current?.focus();
    }).then((unsub) => {
      if (cancelled) unsub();
      else off = unsub;
    });
    return () => {
      cancelled = true;
      off?.();
    };
  }, []);

  const pasteFromClipboard = usePasteFromClipboard({
    onText: (text) => setSourceText(text),
    onError: (message) => setPasteError(message),
  });
  const handlePasteFromClipboard = useCallback(() => {
    setPasteError(null);
    void pasteFromClipboard();
  }, [pasteFromClipboard]);

  useEffect(() => {
    if (!pasteError) return;
    const timer = window.setTimeout(() => setPasteError(null), 2500);
    return () => window.clearTimeout(timer);
  }, [pasteError]);

  // Major 4 — output 길이에 따라 popup 높이 조정. monitor 의 80% 를 cap.
  // 480x360 이 기본; output 이 비면 360 으로 복귀.
  useEffect(() => {
    if (typeof window === 'undefined') return;
    let cancelled = false;
    const adjust = async () => {
      try {
        const monitor = await currentMonitor();
        if (cancelled || !monitor) return;
        const scale = monitor.scaleFactor;
        const maxH = (monitor.size.height / scale) * 0.8;
        const charsPerLine = 64;
        const lineHeight = 18;
        const lines = output ? Math.ceil(output.length / charsPerLine) : 0;
        const base = 240;
        const desired = base + Math.max(0, lines) * lineHeight;
        const target = Math.min(Math.max(desired, 360), maxH);
        await getCurrentWindow().setSize(new LogicalSize(480, target));
      } catch {
        // Tauri 외 환경에서 silent.
      }
    };
    void adjust();
    return () => {
      cancelled = true;
    };
  }, [output]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    return applyTheme(themeFor(theme));
  }, [theme]);

  useEffect(() => {
    if (loaded) setModel(activeModel);
  }, [loaded, activeModel, setModel]);

  const handleClose = useCallback(() => {
    invoke<void>('hide_popup').catch(() => undefined);
  }, []);

  const setCopyError = useTranslationStore((s) => s.setCopyError);
  const handleCopy = useCallback(async () => {
    if (!output) return;
    try {
      await copyText(output);
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

  // Esc / Cmd+C / Cmd+Enter 글로벌 핸들러. Cmd+Enter 는 완료된 번역을 이력에 저장하고
  // 입력/출력을 비운다 — 팝업 창은 닫지 않고 그대로 유지한다.
  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        handleClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        event.preventDefault();
        void saveAndClear();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'c' && output) {
        // textarea/select 영역에 활성 selection 이 있으면 브라우저 기본 동작에 양보.
        const active = document.activeElement as HTMLElement | null;
        if (active instanceof HTMLTextAreaElement || active instanceof HTMLInputElement) {
          const start = active.selectionStart ?? 0;
          const end = active.selectionEnd ?? 0;
          if (end > start) return;
        }
        if (window.getSelection()?.toString()) return;
        event.preventDefault();
        void handleCopy();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [handleClose, handleCopy, output, saveAndClear]);

  const charCount = [...sourceText].length;
  const overLimit = charCount > POPUP_INPUT_LIMIT;

  return (
    <main className="flex h-screen flex-col gap-2 rounded-xl bg-white/90 p-4 text-neutral-900 backdrop-blur-2xl dark:bg-neutral-950/90 dark:text-neutral-100">
      <header className="flex items-center justify-between">
        <h1 className="text-xs font-medium tracking-tight text-neutral-700 dark:text-neutral-300">
          {t('popup.title')}
        </h1>
        <button
          type="button"
          onClick={handleClose}
          aria-label={t('popup.action.close')}
          className="inline-flex size-6 items-center justify-center rounded-md text-neutral-500 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
        >
          <X className="size-3.5" aria-hidden />
        </button>
      </header>

      <div className="flex items-center justify-between">
        <SourceLanguageSelect
          value={sourceLanguage}
          onChange={setSourceLanguage}
          resolvedLanguage={resolvedLanguage}
        />
        <span
          className={
            overLimit
              ? 'font-mono text-[10px] text-rose-600 dark:text-rose-400'
              : 'font-mono text-[10px] text-neutral-500 dark:text-neutral-500'
          }
        >
          {t('translation.input.charCount', { count: charCount, limit: POPUP_INPUT_LIMIT })}
        </span>
      </div>

      <textarea
        ref={textareaRef}
        value={sourceText}
        onChange={(event) => setSourceText(event.target.value)}
        placeholder={t('popup.input.placeholder')}
        spellCheck={false}
        className="h-24 resize-none rounded-md border border-neutral-200 bg-white/70 p-2 text-sm leading-relaxed text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
      />

      <div className="flex items-center justify-between">
        {status === 'translating' ? (
          <span className="inline-flex items-center gap-1 text-[10px] text-neutral-500 dark:text-neutral-400">
            <Loader2 className="size-3 animate-spin" aria-hidden />
            {t('translation.status.translating')}
          </span>
        ) : (
          <span className="text-[10px] text-neutral-400 dark:text-neutral-500">
            {t('popup.shortcuts.hint')}
          </span>
        )}
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={handlePasteFromClipboard}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white/70 px-2 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900/70 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <ClipboardPaste className="size-3" aria-hidden />
            {t('menubar.action.copyClipboard')}
          </button>
          <button
            type="button"
            onClick={handleCopy}
            disabled={!output}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white/70 px-2 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900/70 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            {copied ? (
              <>
                <Check className="size-3" aria-hidden />
                {t('popup.action.copied')}
              </>
            ) : (
              <>
                <Copy className="size-3" aria-hidden />
                {t('popup.action.copy')}
              </>
            )}
          </button>
        </div>
      </div>

      {pasteError ? (
        <div
          role="alert"
          className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-[10px] text-amber-900 dark:border-amber-700 dark:bg-amber-950/40 dark:text-amber-100"
        >
          {pasteError}
        </div>
      ) : null}

      {error ? (
        <div
          role="alert"
          className="rounded-md border border-amber-300 bg-amber-50 p-2 text-xs text-amber-900 dark:border-amber-700 dark:bg-amber-900/30 dark:text-amber-100"
        >
          {messageFor(error)}
        </div>
      ) : (
        <div
          role="status"
          aria-live="polite"
          className="flex-1 overflow-auto whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50/80 p-2 text-sm leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
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
    </main>
  );
}

function themeFor(theme: 'System' | 'Light' | 'Dark'): ThemeMode {
  switch (theme) {
    case 'System':
      return 'system';
    case 'Light':
      return 'light';
    case 'Dark':
      return 'dark';
  }
}

const root = document.getElementById('root');
if (!root) {
  throw new Error('root element missing');
}
createRoot(root).render(<PopupApp />);
