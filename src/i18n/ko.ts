export const ko: Readonly<Record<string, string>> = Object.freeze({});

export type I18nKey = keyof typeof ko;

export function t(key: string): string {
  return ko[key] ?? key;
}
