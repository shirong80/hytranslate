import { createRoot } from 'react-dom/client';

import { TranslationPanel } from '@features/translation/components/translation-panel';
import { applyTheme } from '@lib/theme';

import '@styles/globals.css';

function App() {
  return (
    <main className="h-screen bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <TranslationPanel />
    </main>
  );
}

applyTheme('system');

const root = document.getElementById('root');
if (!root) {
  throw new Error('root element missing');
}
createRoot(root).render(<App />);
