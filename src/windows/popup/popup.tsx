import { createRoot } from 'react-dom/client';

import '@styles/globals.css';

function PopupApp() {
  return (
    <main className="flex h-screen items-center justify-center bg-white/90 text-neutral-900 backdrop-blur dark:bg-neutral-950/90 dark:text-neutral-100">
      <p className="text-sm">HyTranslate Popup</p>
    </main>
  );
}

const root = document.getElementById('root');
if (!root) {
  throw new Error('root element missing');
}
createRoot(root).render(<PopupApp />);
