import { AlertTriangle, RefreshCcw } from 'lucide-react';

import { t } from '@i18n/ko';
import { messageFor, type AppError } from '@lib/ipc/errors';

interface InlineErrorProps {
  error: AppError;
  onRetry?: () => void;
  onOpenOllamaDownload?: () => void;
}

export function InlineError({ error, onRetry, onOpenOllamaDownload }: InlineErrorProps) {
  const showDownload = error.kind === 'OllamaUnavailable';
  const showRetry = onRetry && (error.kind === 'OllamaNotRunning' || error.kind === 'Internal');

  return (
    <div
      role="alert"
      className="flex flex-col gap-3 rounded-md border border-amber-300 bg-amber-50 p-4 text-sm text-amber-900 dark:border-amber-700 dark:bg-amber-900/30 dark:text-amber-100"
    >
      <div className="flex items-start gap-2">
        <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden />
        <p className="leading-relaxed">{messageFor(error)}</p>
      </div>
      {(showDownload || showRetry) && (
        <div className="flex flex-wrap gap-2">
          {showDownload && (
            <button
              type="button"
              onClick={onOpenOllamaDownload}
              className="rounded-md border border-amber-400 bg-white px-3 py-1.5 text-xs font-medium text-amber-900 hover:bg-amber-100 dark:border-amber-600 dark:bg-amber-900/40 dark:text-amber-50 dark:hover:bg-amber-900/60"
            >
              {t('errors.action.openOllamaDownload')}
            </button>
          )}
          {showRetry && (
            <button
              type="button"
              onClick={onRetry}
              className="inline-flex items-center gap-1.5 rounded-md border border-amber-400 bg-white px-3 py-1.5 text-xs font-medium text-amber-900 hover:bg-amber-100 dark:border-amber-600 dark:bg-amber-900/40 dark:text-amber-50 dark:hover:bg-amber-900/60"
            >
              <RefreshCcw className="size-3.5" aria-hidden />
              {t('errors.action.retry')}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
