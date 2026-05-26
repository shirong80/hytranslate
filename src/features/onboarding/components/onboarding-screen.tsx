import {
  AlertCircle,
  ArrowRight,
  CheckCircle2,
  Cpu,
  DownloadCloud,
  ExternalLink,
  HardDrive,
  KeyRound,
  Loader2,
  RefreshCw,
  Save,
  ShieldCheck,
  Sparkles,
} from 'lucide-react';
import { useEffect } from 'react';

import { t } from '@i18n/ko';
import { messageFor } from '@lib/ipc/errors';

import { openOllamaDownloadPage } from '../ipc';
import { useOnboardingStore, type PullProgressView } from '../store';
import { HY_MT2_1_8B, HY_MT2_7B, ONBOARDING_STEPS, type OnboardingStep } from '../types';

interface OnboardingScreenProps {
  onFinished: () => void;
}

const STEP_TITLE_KEY: Record<OnboardingStep, string> = {
  welcome: 'onboarding.step.welcome',
  environment: 'onboarding.step.environment',
  ollama: 'onboarding.step.ollama',
  model: 'onboarding.step.model',
  permissions: 'onboarding.step.permissions',
  history: 'onboarding.step.history',
  done: 'onboarding.step.done',
};

export function OnboardingScreen({ onFinished }: OnboardingScreenProps) {
  const step = useOnboardingStore((s) => s.step);
  const env = useOnboardingStore((s) => s.env);
  const ollama = useOnboardingStore((s) => s.ollama);
  const loadEnvironment = useOnboardingStore((s) => s.loadEnvironment);
  const refreshOllama = useOnboardingStore((s) => s.refreshOllamaStatus);
  const bindListeners = useOnboardingStore((s) => s.bindEventListeners);

  // 환경/Ollama 상태는 진입 시 한 번 자동 fetch.
  useEffect(() => {
    void loadEnvironment();
    void refreshOllama();
  }, [loadEnvironment, refreshOllama]);

  // model-pull 이벤트는 onboarding 진입 동안 살아 있어야 한다.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    void bindListeners().then((off) => {
      unlisten = off;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [bindListeners]);

  // 'done' 도달 시 메인으로 이양.
  useEffect(() => {
    if (step === 'done') onFinished();
  }, [step, env, ollama, onFinished]);

  return (
    <div className="flex h-full flex-col bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <header className="border-b border-neutral-200 bg-white px-8 py-4 dark:border-neutral-800 dark:bg-neutral-900">
        <h1 className="text-lg font-semibold tracking-tight">{t('onboarding.title')}</h1>
        <StepIndicator current={step} />
      </header>
      <main className="flex flex-1 items-start justify-center overflow-auto px-8 py-8">
        <div className="w-full max-w-2xl">
          <StepContent step={step} />
        </div>
      </main>
    </div>
  );
}

function StepIndicator({ current }: { current: OnboardingStep }) {
  const currentIdx = ONBOARDING_STEPS.indexOf(current);
  return (
    <ol className="mt-3 flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400">
      {ONBOARDING_STEPS.filter((s) => s !== 'done').map((step, idx) => {
        const reached = idx <= currentIdx;
        return (
          <li key={step} className="flex items-center gap-2">
            <span
              className={`inline-flex size-5 items-center justify-center rounded-full text-[10px] font-medium ${
                reached
                  ? 'bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900'
                  : 'bg-neutral-200 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400'
              }`}
            >
              {idx + 1}
            </span>
            <span className={reached ? 'text-neutral-700 dark:text-neutral-200' : ''}>
              {t(STEP_TITLE_KEY[step] as Parameters<typeof t>[0])}
            </span>
            {idx < ONBOARDING_STEPS.length - 2 ? (
              <ArrowRight className="size-3 text-neutral-300 dark:text-neutral-700" aria-hidden />
            ) : null}
          </li>
        );
      })}
    </ol>
  );
}

function StepContent({ step }: { step: OnboardingStep }) {
  switch (step) {
    case 'welcome':
      return <WelcomeStep />;
    case 'environment':
      return <EnvironmentStep />;
    case 'ollama':
      return <OllamaStep />;
    case 'model':
      return <ModelStep />;
    case 'permissions':
      return <PermissionsStep />;
    case 'history':
      return <HistoryStep />;
    case 'done':
      return null;
  }
}

function StepCard({
  icon,
  title,
  description,
  children,
  primary,
  secondary,
  error,
}: {
  icon: React.ReactNode;
  title: string;
  description?: string;
  children?: React.ReactNode;
  primary: { label: string; onClick: () => void; disabled?: boolean };
  secondary?: { label: string; onClick: () => void };
  error?: string | null;
}) {
  return (
    <div className="rounded-xl border border-neutral-200 bg-white p-6 shadow-sm dark:border-neutral-800 dark:bg-neutral-900">
      <div className="flex items-start gap-3">
        <div className="rounded-lg bg-neutral-100 p-2 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300">
          {icon}
        </div>
        <div className="flex-1">
          <h2 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">
            {title}
          </h2>
          {description ? (
            <p className="mt-1 text-sm text-neutral-600 dark:text-neutral-400">{description}</p>
          ) : null}
        </div>
      </div>
      {children ? <div className="mt-5">{children}</div> : null}
      {error ? (
        <div className="mt-5 flex items-start gap-2 rounded-md border border-rose-300 bg-rose-50 px-3 py-2 text-sm text-rose-700 dark:border-rose-900 dark:bg-rose-950 dark:text-rose-300">
          <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
          <span>{error}</span>
        </div>
      ) : null}
      <div className="mt-6 flex items-center justify-end gap-2">
        {secondary ? (
          <button
            type="button"
            onClick={secondary.onClick}
            className="inline-flex items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-sm text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            {secondary.label}
          </button>
        ) : null}
        <button
          type="button"
          onClick={primary.onClick}
          disabled={primary.disabled}
          className="inline-flex items-center gap-1.5 rounded-md bg-neutral-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-white"
        >
          {primary.label}
          <ArrowRight className="size-3.5" aria-hidden />
        </button>
      </div>
    </div>
  );
}

function WelcomeStep() {
  const goNext = useOnboardingStore((s) => s.goNext);
  return (
    <StepCard
      icon={<Sparkles className="size-5" aria-hidden />}
      title={t('onboarding.welcome.title')}
      description={t('onboarding.welcome.description')}
      primary={{ label: t('onboarding.welcome.start'), onClick: goNext }}
    >
      <ul className="space-y-2 text-sm text-neutral-700 dark:text-neutral-300">
        <li className="flex items-start gap-2">
          <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-600" aria-hidden />
          <span>{t('onboarding.welcome.bullet.local')}</span>
        </li>
        <li className="flex items-start gap-2">
          <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-600" aria-hidden />
          <span>{t('onboarding.welcome.bullet.offline')}</span>
        </li>
        <li className="flex items-start gap-2">
          <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-600" aria-hidden />
          <span>{t('onboarding.welcome.bullet.history')}</span>
        </li>
      </ul>
    </StepCard>
  );
}

function EnvironmentStep() {
  const env = useOnboardingStore((s) => s.env);
  const loading = useOnboardingStore((s) => s.loading);
  const error = useOnboardingStore((s) => s.error);
  const loadEnvironment = useOnboardingStore((s) => s.loadEnvironment);
  const goNext = useOnboardingStore((s) => s.goNext);
  const goPrev = useOnboardingStore((s) => s.goPrev);

  return (
    <StepCard
      icon={<Cpu className="size-5" aria-hidden />}
      title={t('onboarding.environment.title')}
      description={t('onboarding.environment.description')}
      primary={{ label: t('onboarding.action.continue'), onClick: goNext, disabled: loading }}
      secondary={{ label: t('onboarding.action.back'), onClick: goPrev }}
      error={error ? messageFor(error) : null}
    >
      {loading || !env ? (
        <div className="flex items-center gap-2 text-sm text-neutral-500">
          <Loader2 className="size-4 animate-spin" aria-hidden />
          {t('onboarding.environment.checking')}
        </div>
      ) : (
        <div className="space-y-2 text-sm">
          <Row
            label={t('onboarding.environment.macos')}
            value={env.macosVersion}
            warn={!env.macosSupported}
            warnText={t('onboarding.environment.macosUnsupported')}
          />
          <Row
            label={t('onboarding.environment.arch')}
            value={
              env.arch === 'AppleSilicon'
                ? t('onboarding.environment.arch.appleSilicon')
                : env.arch === 'Intel'
                  ? t('onboarding.environment.arch.intel')
                  : t('onboarding.environment.arch.unknown')
            }
            warn={env.arch === 'Intel'}
            warnText={t('onboarding.environment.intelWarning')}
          />
          <Row
            label={t('onboarding.environment.memory')}
            value={env.totalMemoryGb > 0 ? `${env.totalMemoryGb} GB` : '—'}
            warn={env.totalMemoryGb > 0 && env.totalMemoryGb < 12}
            warnText={t('onboarding.environment.lowMemory')}
          />
          <div className="pt-2">
            <button
              type="button"
              onClick={() => void loadEnvironment()}
              className="inline-flex items-center gap-1.5 text-xs text-neutral-500 hover:text-neutral-700 dark:text-neutral-400 dark:hover:text-neutral-200"
            >
              <RefreshCw className="size-3" aria-hidden />
              {t('onboarding.action.recheck')}
            </button>
          </div>
        </div>
      )}
    </StepCard>
  );
}

function OllamaStep() {
  const ollama = useOnboardingStore((s) => s.ollama);
  const loading = useOnboardingStore((s) => s.loading);
  const error = useOnboardingStore((s) => s.error);
  const refresh = useOnboardingStore((s) => s.refreshOllamaStatus);
  const goNext = useOnboardingStore((s) => s.goNext);
  const goPrev = useOnboardingStore((s) => s.goPrev);

  const running = ollama?.running === true;

  return (
    <StepCard
      icon={<HardDrive className="size-5" aria-hidden />}
      title={t('onboarding.ollama.title')}
      description={t('onboarding.ollama.description')}
      primary={{
        label: t('onboarding.action.continue'),
        onClick: goNext,
        disabled: !running,
      }}
      secondary={{ label: t('onboarding.action.back'), onClick: goPrev }}
      error={error ? messageFor(error) : null}
    >
      <div className="space-y-3 text-sm">
        <div
          className={`flex items-start gap-2 rounded-md border px-3 py-2 ${
            running
              ? 'border-emerald-300 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300'
              : 'border-amber-300 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-300'
          }`}
        >
          {running ? (
            <CheckCircle2 className="mt-0.5 size-4 shrink-0" aria-hidden />
          ) : (
            <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
          )}
          <span>
            {running ? t('onboarding.ollama.running') : t('onboarding.ollama.notRunning')}
          </span>
        </div>
        <div className="text-xs text-neutral-500 dark:text-neutral-400">
          endpoint: <code>{ollama?.endpoint ?? '—'}</code>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={() => void refresh()}
            disabled={loading}
            className="inline-flex items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            {loading ? (
              <Loader2 className="size-3 animate-spin" aria-hidden />
            ) : (
              <RefreshCw className="size-3" aria-hidden />
            )}
            {t('onboarding.action.recheck')}
          </button>
          {!running ? (
            <button
              type="button"
              onClick={() => void openOllamaDownloadPage()}
              className="inline-flex items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-3 py-1.5 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
            >
              <ExternalLink className="size-3" aria-hidden />
              {t('onboarding.ollama.openDownload')}
            </button>
          ) : null}
        </div>
      </div>
    </StepCard>
  );
}

function ModelStep() {
  const env = useOnboardingStore((s) => s.env);
  const ollama = useOnboardingStore((s) => s.ollama);
  const selectedModel = useOnboardingStore((s) => s.selectedModel);
  const selectModel = useOnboardingStore((s) => s.selectModel);
  const pullingModel = useOnboardingStore((s) => s.pullingModel);
  const progress = useOnboardingStore((s) => s.progress);
  const installedSinceStart = useOnboardingStore((s) => s.installedSinceStart);
  const startPull = useOnboardingStore((s) => s.startPull);
  const cancelPull = useOnboardingStore((s) => s.cancelPull);
  const refreshOllama = useOnboardingStore((s) => s.refreshOllamaStatus);
  const error = useOnboardingStore((s) => s.error);
  const goNext = useOnboardingStore((s) => s.goNext);
  const goPrev = useOnboardingStore((s) => s.goPrev);

  const recommended = env?.recommendedModel ?? HY_MT2_7B;
  const installedModels = new Set([...(ollama?.models ?? []), ...installedSinceStart]);
  const isInstalled = installedModels.has(selectedModel);
  const canContinue = isInstalled;

  // pull 완료 직후 ollama 모델 리스트 동기화. installedSinceStart 가 추가되면 한 번 갱신.
  useEffect(() => {
    if (installedSinceStart.length === 0) return;
    void refreshOllama();
  }, [installedSinceStart, refreshOllama]);

  return (
    <StepCard
      icon={<DownloadCloud className="size-5" aria-hidden />}
      title={t('onboarding.model.title')}
      description={t('onboarding.model.description')}
      primary={{
        label: t('onboarding.action.continue'),
        onClick: goNext,
        disabled: !canContinue || pullingModel !== null,
      }}
      secondary={{ label: t('onboarding.action.back'), onClick: goPrev }}
      error={error ? messageFor(error) : null}
    >
      <div className="space-y-4">
        <fieldset className="space-y-2">
          <legend className="text-xs font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400">
            {t('onboarding.model.choose')}
          </legend>
          <ModelOption
            value={HY_MT2_7B}
            label={t('onboarding.model.hy7b.label')}
            sub={t('onboarding.model.hy7b.sub')}
            checked={selectedModel === HY_MT2_7B}
            recommended={recommended === HY_MT2_7B}
            installed={installedModels.has(HY_MT2_7B)}
            onChange={() => selectModel(HY_MT2_7B)}
            disabled={pullingModel !== null}
          />
          <ModelOption
            value={HY_MT2_1_8B}
            label={t('onboarding.model.hy1_8b.label')}
            sub={t('onboarding.model.hy1_8b.sub')}
            checked={selectedModel === HY_MT2_1_8B}
            recommended={recommended === HY_MT2_1_8B}
            installed={installedModels.has(HY_MT2_1_8B)}
            onChange={() => selectModel(HY_MT2_1_8B)}
            disabled={pullingModel !== null}
          />
        </fieldset>

        {pullingModel ? (
          <PullProgress
            model={pullingModel}
            progress={progress}
            onCancel={() => void cancelPull()}
          />
        ) : isInstalled ? (
          <div className="flex items-start gap-2 rounded-md border border-emerald-300 bg-emerald-50 px-3 py-2 text-sm text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300">
            <CheckCircle2 className="mt-0.5 size-4 shrink-0" aria-hidden />
            <span>{t('onboarding.model.alreadyInstalled')}</span>
          </div>
        ) : (
          <button
            type="button"
            onClick={() => void startPull(selectedModel)}
            disabled={ollama?.running !== true}
            className="inline-flex items-center gap-1.5 rounded-md bg-neutral-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-white"
          >
            <DownloadCloud className="size-3.5" aria-hidden />
            {t('onboarding.model.startPull')}
          </button>
        )}
      </div>
    </StepCard>
  );
}

function ModelOption({
  value,
  label,
  sub,
  checked,
  recommended,
  installed,
  onChange,
  disabled,
}: {
  value: string;
  label: string;
  sub: string;
  checked: boolean;
  recommended: boolean;
  installed: boolean;
  onChange: () => void;
  disabled: boolean;
}) {
  return (
    <label
      htmlFor={`onboarding-model-${value}`}
      className={`flex cursor-pointer items-start gap-3 rounded-md border px-3 py-2.5 text-sm transition ${
        checked
          ? 'border-neutral-900 bg-neutral-50 dark:border-neutral-100 dark:bg-neutral-800'
          : 'border-neutral-200 hover:border-neutral-300 dark:border-neutral-800 dark:hover:border-neutral-700'
      } ${disabled ? 'cursor-not-allowed opacity-60' : ''}`}
    >
      <input
        id={`onboarding-model-${value}`}
        type="radio"
        name="onboarding-model"
        value={value}
        checked={checked}
        onChange={onChange}
        disabled={disabled}
        className="mt-1 size-3.5"
        aria-label={label}
      />
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium text-neutral-900 dark:text-neutral-100">{label}</span>
          {recommended ? (
            <span className="rounded-full bg-neutral-900 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-white dark:bg-neutral-100 dark:text-neutral-900">
              {t('onboarding.model.recommended')}
            </span>
          ) : null}
          {installed ? (
            <span className="rounded-full bg-emerald-100 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-emerald-700 dark:bg-emerald-900 dark:text-emerald-300">
              {t('onboarding.model.installed')}
            </span>
          ) : null}
        </div>
        <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{sub}</p>
      </div>
    </label>
  );
}

function PullProgress({
  model,
  progress,
  onCancel,
}: {
  model: string;
  progress: PullProgressView | null;
  onCancel: () => void;
}) {
  const pct =
    progress?.ratio !== null && progress?.ratio !== undefined
      ? Math.round(progress.ratio * 100)
      : null;
  return (
    <div className="rounded-md border border-neutral-200 bg-neutral-50 px-3 py-3 text-sm dark:border-neutral-800 dark:bg-neutral-900">
      <div className="flex items-center gap-2">
        <Loader2
          className="size-4 animate-spin text-neutral-600 dark:text-neutral-300"
          aria-hidden
        />
        <span className="font-medium">{t('onboarding.model.pulling', { model })}</span>
      </div>
      <div className="mt-2 text-xs text-neutral-500 dark:text-neutral-400">
        {progress?.status ?? '—'}
        {pct !== null ? ` · ${pct}%` : ''}
        {progress?.completed !== null &&
        progress?.completed !== undefined &&
        progress?.total !== null &&
        progress?.total !== undefined
          ? ` · ${formatBytes(progress.completed)} / ${formatBytes(progress.total)}`
          : ''}
      </div>
      <div className="mt-2 h-1.5 overflow-hidden rounded bg-neutral-200 dark:bg-neutral-800">
        <div
          className="h-full bg-neutral-900 transition-all dark:bg-neutral-100"
          style={{ width: pct !== null ? `${pct}%` : '12%' }}
        />
      </div>
      <div className="mt-3 flex justify-end">
        <button
          type="button"
          onClick={onCancel}
          className="inline-flex items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-2.5 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
        >
          {t('onboarding.action.cancel')}
        </button>
      </div>
    </div>
  );
}

function PermissionsStep() {
  const goNext = useOnboardingStore((s) => s.goNext);
  const goPrev = useOnboardingStore((s) => s.goPrev);
  return (
    <StepCard
      icon={<KeyRound className="size-5" aria-hidden />}
      title={t('onboarding.permissions.title')}
      description={t('onboarding.permissions.description')}
      primary={{ label: t('onboarding.action.continue'), onClick: goNext }}
      secondary={{ label: t('onboarding.action.back'), onClick: goPrev }}
    >
      <ul className="space-y-2 text-sm text-neutral-700 dark:text-neutral-300">
        <li className="flex items-start gap-2">
          <ShieldCheck className="mt-0.5 size-4 shrink-0 text-neutral-500" aria-hidden />
          <span>{t('onboarding.permissions.accessibility')}</span>
        </li>
        <li className="flex items-start gap-2">
          <ShieldCheck className="mt-0.5 size-4 shrink-0 text-neutral-500" aria-hidden />
          <span>{t('onboarding.permissions.note')}</span>
        </li>
      </ul>
    </StepCard>
  );
}

function HistoryStep() {
  const finish = useOnboardingStore((s) => s.finish);
  const error = useOnboardingStore((s) => s.error);
  const loading = useOnboardingStore((s) => s.loading);
  const goPrev = useOnboardingStore((s) => s.goPrev);
  return (
    <StepCard
      icon={<Save className="size-5" aria-hidden />}
      title={t('onboarding.history.title')}
      description={t('onboarding.history.description')}
      primary={{
        label: loading ? t('onboarding.action.finishing') : t('onboarding.action.finish'),
        onClick: () => void finish(),
        disabled: loading,
      }}
      secondary={{ label: t('onboarding.action.back'), onClick: goPrev }}
      error={error ? messageFor(error) : null}
    />
  );
}

function Row({
  label,
  value,
  warn,
  warnText,
}: {
  label: string;
  value: string;
  warn?: boolean;
  warnText?: string;
}) {
  return (
    <div className="flex items-start justify-between gap-4 border-b border-neutral-100 pb-2 last:border-0 last:pb-0 dark:border-neutral-800">
      <span className="text-neutral-500 dark:text-neutral-400">{label}</span>
      <div className="text-right">
        <span className="font-medium text-neutral-900 dark:text-neutral-100">{value}</span>
        {warn && warnText ? (
          <p className="mt-0.5 text-xs text-amber-600 dark:text-amber-400">{warnText}</p>
        ) : null}
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}
