import { Languages } from 'lucide-react';

import { t } from '@i18n/ko';

import { type SourceLanguage } from '../types';

interface SourceLanguageSelectProps {
  value: SourceLanguage;
  onChange: (lang: SourceLanguage) => void;
  /**
   * Auto 입력에서 backend 가 detect 한 언어. translation:started 이벤트로 도착.
   * value === 'Auto' 일 때만 badge 가 렌더되며, null 이면 미감지 (idle/typing).
   */
  resolvedLanguage?: SourceLanguage | null;
}

export function SourceLanguageSelect({
  value,
  onChange,
  resolvedLanguage,
}: SourceLanguageSelectProps) {
  return (
    <div className="flex items-center gap-2">
      <label className="flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-400">
        <Languages className="size-3.5" aria-hidden />
        <span className="sr-only">{t('translation.sourceLanguage.label')}</span>
        <select
          value={value}
          onChange={(event) => {
            onChange(event.target.value as SourceLanguage);
          }}
          aria-label={t('translation.sourceLanguage.label')}
          className="rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-900 hover:border-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100 dark:hover:border-neutral-600"
        >
          <option value="Auto">{t('translation.sourceLanguage.auto')}</option>
          <option value="Korean">{t('translation.sourceLanguage.korean')}</option>
          <option value="ChineseSimplified">
            {t('translation.sourceLanguage.chineseSimplified')}
          </option>
          <option value="ChineseTraditional">
            {t('translation.sourceLanguage.chineseTraditional')}
          </option>
        </select>
      </label>
      {value === 'Auto' && resolvedLanguage ? (
        <DetectedBadge resolvedLanguage={resolvedLanguage} />
      ) : null}
    </div>
  );
}

function DetectedBadge({ resolvedLanguage }: { resolvedLanguage: SourceLanguage }) {
  return (
    <span
      role="status"
      className="rounded-md bg-neutral-100 px-1.5 py-0.5 text-[10px] font-medium text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300"
    >
      {detectedLabel(resolvedLanguage)}
    </span>
  );
}

function detectedLabel(language: SourceLanguage): string {
  switch (language) {
    case 'Korean':
      return t('translation.sourceLanguage.detected.korean');
    case 'ChineseSimplified':
      return t('translation.sourceLanguage.detected.chineseSimplified');
    case 'ChineseTraditional':
      return t('translation.sourceLanguage.detected.chineseTraditional');
    default:
      return t('translation.sourceLanguage.detected.unknown');
  }
}
