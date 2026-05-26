export type ThemeMode = 'system' | 'light' | 'dark';

const DARK_QUERY = '(prefers-color-scheme: dark)';

function setHtmlDarkClass(isDark: boolean): void {
  const root = document.documentElement;
  if (isDark) root.classList.add('dark');
  else root.classList.remove('dark');
}

/**
 * 시스템 light/dark/system 테마를 `<html>` 의 `dark` 클래스로 반영한다.
 * 반환되는 함수를 호출하면 listener 를 정리한다.
 */
export function applyTheme(mode: ThemeMode): () => void {
  const mql = window.matchMedia(DARK_QUERY);

  const apply = () => {
    if (mode === 'dark') setHtmlDarkClass(true);
    else if (mode === 'light') setHtmlDarkClass(false);
    else setHtmlDarkClass(mql.matches);
  };

  apply();

  if (mode !== 'system') return () => {};

  const handler = (event: MediaQueryListEvent) => setHtmlDarkClass(event.matches);
  mql.addEventListener('change', handler);
  return () => mql.removeEventListener('change', handler);
}
