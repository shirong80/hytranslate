import { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';

import { SettingsPanel } from '@features/settings/components/settings-panel';
import { useSettingsStore } from '@features/settings/store';
import { TranslationPanel } from '@features/translation/components/translation-panel';
import { useTranslationStore } from '@features/translation/store';
import { applyTheme, type ThemeMode } from '@lib/theme';

import '@styles/globals.css';

type Route = 'translate' | 'settings';

function App() {
  const [route, setRoute] = useState<Route>('translate');

  // Settings 부트스트랩 + 테마/모델 동기화.
  const load = useSettingsStore((s) => s.load);
  const loaded = useSettingsStore((s) => s.loaded);
  const theme = useSettingsStore((s) => s.settings.theme);
  const activeModel = useSettingsStore((s) => s.settings.activeModel);
  const setModel = useTranslationStore((s) => s.setModel);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    return applyTheme(themeFor(theme));
  }, [theme]);

  useEffect(() => {
    if (loaded) setModel(activeModel);
  }, [loaded, activeModel, setModel]);

  return (
    <main className="h-screen bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      {route === 'translate' ? (
        <TranslationPanel onOpenSettings={() => setRoute('settings')} />
      ) : (
        <SettingsPanel onBack={() => setRoute('translate')} />
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
createRoot(root).render(<App />);
