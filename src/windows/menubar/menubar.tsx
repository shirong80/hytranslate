import { createRoot } from 'react-dom/client';

import '@styles/globals.css';

function MenubarApp() {
  return (
    <main className="flex h-screen items-center justify-center bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <p className="text-sm">HyTranslate Menubar</p>
    </main>
  );
}

const root = document.getElementById('root');
if (!root) {
  throw new Error('root element missing');
}
createRoot(root).render(<MenubarApp />);
