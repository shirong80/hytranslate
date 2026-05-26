import { Languages } from 'lucide-react';

import { t } from '@i18n/ko';

import { type SourceLanguage } from '../types';

interface SourceLanguageSelectProps {
  value: SourceLanguage;
  onChange: (lang: SourceLanguage) => void;
}

export function SourceLanguageSelect({ value, onChange }: SourceLanguageSelectProps) {
  return (
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
  );
}
