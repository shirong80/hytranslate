import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { ClipboardPaste, Loader2 } from 'lucide-react';
import { useCallback, useEffect, useRef } from 'react';
import { createRoot } from 'react-dom/client';

import { useSettingsStore } from '@features/settings/store';
import { useTranslationStore } from '@features/translation/store';
import { MENUBAR_INPUT_LIMIT } from '@features/translation/types';
import { useTranslationController } from '@features/translation/use-translation-controller';
import { t } from '@i18n/ko';
import { useAutoCopyTranslation } from '@lib/hooks/use-auto-copy-translation';
import { applyTheme, type ThemeMode } from '@lib/theme';

import '@styles/globals.css';

function MenubarApp() {
  const load = useSettingsStore((s) => s.load);
  const loaded = useSettingsStore((s) => s.loaded);
  const theme = useSettingsStore((s) => s.settings.theme);
  const activeModel = useSettingsStore((s) => s.settings.activeModel);
  const autoCopy = useSettingsStore((s) => s.settings.autoCopyAfterTranslation);

  const sourceText = useTranslationStore((s) => s.sourceText);
  const output = useTranslationStore((s) => s.output);
  const status = useTranslationStore((s) => s.status);
  const recent = useTranslationStore((s) => s.recent);

  const setSourceText = useTranslationStore((s) => s.setSourceText);
  const setModel = useTranslationStore((s) => s.setModel);

  useTranslationController({ inputLimit: MENUBAR_INPUT_LIMIT });
  useAutoCopyTranslation(autoCopy);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // 메뉴바 popover 가 열릴 때 자동 포커스. autoFocus prop 대신 ref 로 명시.
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

  const handlePasteFromClipboard = useCallback(async () => {
    try {
      const text = await readText();
      if (text) setSourceText(text);
    } catch {
      // Tauri 외 환경에서는 무시.
    }
  }, [setSourceText]);

  const charCount = [...sourceText].length;
  const overLimit = charCount > MENUBAR_INPUT_LIMIT;

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

      <div
        role="status"
        aria-live="polite"
        className="min-h-16 max-h-32 overflow-auto whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50/80 p-2 text-xs leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100"
      >
        {output || (
          <span className="text-neutral-400 dark:text-neutral-600">
            {t('translation.output.placeholder')}
          </span>
        )}
      </div>

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
                key={r.requestId}
                className="rounded-md border border-neutral-200 bg-white/60 px-2 py-1 dark:border-neutral-800 dark:bg-neutral-900/50"
              >
                <p className="line-clamp-1 text-[10px] text-neutral-500 dark:text-neutral-500">
                  {r.sourceText}
                </p>
                <p className="line-clamp-1 text-xs text-neutral-800 dark:text-neutral-200">
                  {r.fullText}
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
