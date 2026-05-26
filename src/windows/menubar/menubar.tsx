import { ClipboardPaste, Loader2 } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';

import { usePasteFromClipboard } from '@features/clipboard/hooks';
import { listTranslationRecords } from '@features/history/ipc';
import type { TranslationRecord } from '@features/history/types';
import { useSettingsStore } from '@features/settings/store';
import { useTranslationStore } from '@features/translation/store';
import { MENUBAR_INPUT_LIMIT } from '@features/translation/types';
import { useTranslationController } from '@features/translation/use-translation-controller';
import { t } from '@i18n/ko';
import { useAutoCopyTranslation } from '@lib/hooks/use-auto-copy-translation';
import { listen } from '@lib/ipc/client';
import { messageFor } from '@lib/ipc/errors';
import { MENUBAR_OPENED } from '@lib/ipc/events';
import { applyTheme, type ThemeMode } from '@lib/theme';

import '@styles/globals.css';

const RECENT_LIMIT = 5;

function MenubarApp() {
  const load = useSettingsStore((s) => s.load);
  const loaded = useSettingsStore((s) => s.loaded);
  const theme = useSettingsStore((s) => s.settings.theme);
  const activeModel = useSettingsStore((s) => s.settings.activeModel);
  const autoCopy = useSettingsStore((s) => s.settings.autoCopyAfterTranslation);

  const sourceText = useTranslationStore((s) => s.sourceText);
  const output = useTranslationStore((s) => s.output);
  const status = useTranslationStore((s) => s.status);

  const setSourceText = useTranslationStore((s) => s.setSourceText);
  const setModel = useTranslationStore((s) => s.setModel);

  useTranslationController({ inputLimit: MENUBAR_INPUT_LIMIT });
  useAutoCopyTranslation(autoCopy);

  const [recent, setRecent] = useState<TranslationRecord[]>([]);
  const [pasteError, setPasteError] = useState<string | null>(null);

  const refreshRecent = useCallback(async () => {
    try {
      const result = await listTranslationRecords({ limit: RECENT_LIMIT });
      setRecent(result.records);
    } catch {
      // 자동 게이트 회복: DB unavailable 환경이면 빈 리스트 유지.
      setRecent([]);
    }
  }, []);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // 메뉴바 popover 가 열릴 때 자동 포커스.
    textareaRef.current?.focus();
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    return applyTheme(themeFor(theme));
  }, [theme]);

  useEffect(() => {
    if (loaded) setModel(activeModel);
  }, [loaded, activeModel, setModel]);

  // 최초 로드 + popover 재오픈 시 매번 재조회.
  useEffect(() => {
    void refreshRecent();
    let off: (() => void) | undefined;
    let cancelled = false;
    listen<void>(MENUBAR_OPENED, () => {
      void refreshRecent();
    }).then((unsub) => {
      if (cancelled) unsub();
      else off = unsub;
    });
    return () => {
      cancelled = true;
      off?.();
    };
  }, [refreshRecent]);

  const pasteFromClipboard = usePasteFromClipboard({
    onText: (text) => setSourceText(text),
    onError: (message) => setPasteError(message),
  });
  const handlePasteFromClipboard = useCallback(() => {
    setPasteError(null);
    void pasteFromClipboard();
  }, [pasteFromClipboard]);

  const charCount = [...sourceText].length;
  const overLimit = charCount > MENUBAR_INPUT_LIMIT;

  const errorBanner = useTranslationStore((s) => s.error);

  return (
    <main className="flex h-screen flex-col gap-2 bg-white/95 p-3 text-neutral-900 backdrop-blur-2xl dark:bg-neutral-950/95 dark:text-neutral-100">
      <header className="flex items-center justify-between">
        <h1 className="text-xs font-medium tracking-tight text-neutral-700 dark:text-neutral-300">
          {t('app.title')}
        </h1>
        <span
          className={
            overLimit
              ? 'font-mono text-[10px] text-rose-600 dark:text-rose-400'
              : 'font-mono text-[10px] text-neutral-500 dark:text-neutral-500'
          }
        >
          {t('translation.input.charCount', { count: charCount, limit: MENUBAR_INPUT_LIMIT })}
        </span>
      </header>

      {pasteError ? (
        <div
          role="alert"
          className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-[10px] text-amber-900 dark:border-amber-700 dark:bg-amber-950/40 dark:text-amber-100"
        >
          {pasteError}
        </div>
      ) : null}

      <textarea
        ref={textareaRef}
        value={sourceText}
        onChange={(event) => setSourceText(event.target.value)}
        placeholder={t('menubar.input.placeholder')}
        spellCheck={false}
        className="h-16 resize-none rounded-md border border-neutral-200 bg-white/70 p-2 text-xs leading-relaxed text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
      />

      <div className="flex items-center justify-between">
        <button
          type="button"
          onClick={handlePasteFromClipboard}
          className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white/70 px-2 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900/70 dark:text-neutral-300 dark:hover:bg-neutral-800"
        >
          <ClipboardPaste className="size-3" aria-hidden />
          {t('menubar.action.copyClipboard')}
        </button>
        {status === 'translating' ? (
          <span className="inline-flex items-center gap-1 text-[10px] text-neutral-500 dark:text-neutral-400">
            <Loader2 className="size-3 animate-spin" aria-hidden />
            {t('translation.status.translating')}
          </span>
        ) : null}
      </div>

      {errorBanner ? (
        <div
          role="alert"
          className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-[10px] text-amber-900 dark:border-amber-700 dark:bg-amber-950/40 dark:text-amber-100"
        >
          {messageFor(errorBanner)}
        </div>
      ) : (
        <div
          role="status"
          aria-live="polite"
          className="max-h-32 min-h-16 overflow-auto whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50/80 p-2 text-xs leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
        >
          {output || (
            <span className="text-neutral-400 dark:text-neutral-600">
              {t('translation.output.placeholder')}
            </span>
          )}
        </div>
      )}

      <section className="flex flex-col gap-1">
        <h2 className="text-[10px] font-medium uppercase tracking-wider text-neutral-500 dark:text-neutral-500">
          {t('menubar.recent.title')}
        </h2>
        {recent.length === 0 ? (
          <p className="text-[10px] text-neutral-400 dark:text-neutral-600">
            {t('menubar.recent.empty')}
          </p>
        ) : (
          <ul className="flex flex-col gap-1 overflow-auto">
            {recent.map((r) => (
              <li
                key={r.id}
                className="rounded-md border border-neutral-200 bg-white/60 px-2 py-1 dark:border-neutral-800 dark:bg-neutral-900/50"
              >
                <p className="line-clamp-1 text-[10px] text-neutral-500 dark:text-neutral-500">
                  {r.sourceText}
                </p>
                <p className="line-clamp-1 text-xs text-neutral-800 dark:text-neutral-200">
                  {r.translatedText}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>
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
createRoot(root).render(<MenubarApp />);
