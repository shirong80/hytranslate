import {
  ArrowLeft,
  Copy,
  Download,
  FileJson,
  FileSpreadsheet,
  Search,
  Star,
  Trash2,
  X,
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';

import { t } from '@i18n/ko';
import { copyText } from '@lib/clipboard';
import { messageFor } from '@lib/ipc/errors';

import { useHistoryStore } from '../store';
import type { TranslationRecord } from '../types';

interface HistoryPanelProps {
  onBack?: () => void;
}

export function HistoryPanel({ onBack }: HistoryPanelProps = {}) {
  const records = useHistoryStore((s) => s.records);
  const total = useHistoryStore((s) => s.total);
  const loading = useHistoryStore((s) => s.loading);
  const error = useHistoryStore((s) => s.error);
  const query = useHistoryStore((s) => s.query);
  const filters = useHistoryStore((s) => s.filters);
  const selectedId = useHistoryStore((s) => s.selectedId);
  const lastExport = useHistoryStore((s) => s.lastExport);

  const fetchHistory = useHistoryStore((s) => s.fetch);
  const setQuery = useHistoryStore((s) => s.setQuery);
  const setFavoriteFilter = useHistoryStore((s) => s.setFavoriteFilter);
  const setTagFilter = useHistoryStore((s) => s.setTagFilter);
  const selectRecord = useHistoryStore((s) => s.selectRecord);
  const toggleFavorite = useHistoryStore((s) => s.toggleFavorite);
  const setTags = useHistoryStore((s) => s.setTags);
  const removeRecord = useHistoryStore((s) => s.removeRecord);
  const removeAll = useHistoryStore((s) => s.removeAll);
  const exportCsv = useHistoryStore((s) => s.exportCsv);
  const exportJson = useHistoryStore((s) => s.exportJson);

  useEffect(() => {
    void fetchHistory();
  }, [fetchHistory]);

  // 검색어 / 필터 변경 시 300ms 디바운스 후 fetch.
  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchHistory();
    }, 300);
    return () => window.clearTimeout(timer);
  }, [query, filters.favoriteOnly, filters.tag, fetchHistory]);

  const selected = useMemo(
    () => records.find((r) => r.id === selectedId) ?? null,
    [records, selectedId],
  );

  const handleDeleteAll = useCallback(() => {
    if (window.confirm(t('history.deleteAll.confirm'))) {
      void removeAll();
    }
  }, [removeAll]);

  return (
    <div className="flex h-full flex-col gap-3 p-6">
      <header className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {onBack ? (
            <button
              type="button"
              onClick={onBack}
              aria-label={t('nav.back')}
              className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
            >
              <ArrowLeft className="size-3.5" aria-hidden />
              {t('nav.back')}
            </button>
          ) : null}
          <h1 className="text-lg font-medium tracking-tight text-neutral-900 dark:text-neutral-100">
            {t('history.title')}
          </h1>
          <span className="text-xs text-neutral-500 dark:text-neutral-500">
            {t('history.total', { count: total })}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span
            className="text-[10px] text-neutral-500 dark:text-neutral-500"
            title={t('history.export.notice')}
          >
            {t('history.export.notice')}
          </span>
          <button
            type="button"
            onClick={() => void exportCsv()}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <FileSpreadsheet className="size-3.5" aria-hidden />
            {t('history.export.csv')}
          </button>
          <button
            type="button"
            onClick={() => void exportJson()}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <FileJson className="size-3.5" aria-hidden />
            {t('history.export.json')}
          </button>
          <button
            type="button"
            onClick={handleDeleteAll}
            disabled={total === 0}
            className="inline-flex items-center gap-1 rounded-md border border-rose-200 bg-white px-2 py-1 text-xs text-rose-700 hover:border-rose-300 hover:bg-rose-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-rose-900 dark:bg-neutral-900 dark:text-rose-300 dark:hover:bg-rose-950"
          >
            <Trash2 className="size-3.5" aria-hidden />
            {t('history.deleteAll')}
          </button>
        </div>
      </header>

      <FiltersBar
        query={query}
        onQueryChange={setQuery}
        favoriteOnly={filters.favoriteOnly}
        onFavoriteOnlyChange={setFavoriteFilter}
        tag={filters.tag}
        onTagChange={setTagFilter}
      />

      {error ? (
        <p className="rounded-md border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700 dark:border-rose-900 dark:bg-rose-950 dark:text-rose-300">
          {messageFor(error)}
        </p>
      ) : null}
      {lastExport?.path ? (
        <p className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300">
          {t('history.export.success', {
            count: lastExport.records,
            path: lastExport.path,
          })}
        </p>
      ) : null}

      <div className="grid flex-1 grid-cols-1 gap-4 overflow-hidden md:grid-cols-[minmax(280px,_2fr)_3fr]">
        <RecordList
          records={records}
          loading={loading}
          selectedId={selectedId}
          onSelect={selectRecord}
        />
        <DetailPane
          record={selected}
          onToggleFavorite={() => selected && void toggleFavorite(selected.id)}
          onSetTags={(tags) => selected && void setTags(selected.id, tags)}
          onDelete={() => selected && void removeRecord(selected.id)}
        />
      </div>
    </div>
  );
}

function FiltersBar({
  query,
  onQueryChange,
  favoriteOnly,
  onFavoriteOnlyChange,
  tag,
  onTagChange,
}: {
  query: string;
  onQueryChange: (q: string) => void;
  favoriteOnly: boolean;
  onFavoriteOnlyChange: (on: boolean) => void;
  tag: string | null;
  onTagChange: (tag: string | null) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <div className="relative flex-1">
        <Search
          className="pointer-events-none absolute left-2 top-1/2 size-3.5 -translate-y-1/2 text-neutral-400"
          aria-hidden
        />
        <input
          type="search"
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder={t('history.search.placeholder')}
          aria-label={t('history.search.placeholder')}
          spellCheck={false}
          className="w-full rounded-md border border-neutral-300 bg-white py-1.5 pl-7 pr-3 text-xs text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-600"
        />
      </div>
      <input
        type="text"
        value={tag ?? ''}
        onChange={(e) => onTagChange(e.target.value || null)}
        placeholder={t('history.filter.tagPlaceholder')}
        aria-label={t('history.filter.tagPlaceholder')}
        spellCheck={false}
        className="w-32 rounded-md border border-neutral-300 bg-white px-2 py-1.5 text-xs text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-600"
      />
      <label className="inline-flex cursor-pointer items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-2 py-1.5 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800">
        <input
          type="checkbox"
          checked={favoriteOnly}
          onChange={(e) => onFavoriteOnlyChange(e.target.checked)}
          className="size-3.5 cursor-pointer accent-brand"
        />
        <Star className="size-3.5" aria-hidden />
        {t('history.filter.favoriteOnly')}
      </label>
    </div>
  );
}

function RecordList({
  records,
  loading,
  selectedId,
  onSelect,
}: {
  records: TranslationRecord[];
  loading: boolean;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}) {
  if (loading && records.length === 0) {
    return (
      <p className="rounded-md border border-neutral-200 bg-white p-4 text-xs text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-500">
        {t('history.loading')}
      </p>
    );
  }
  if (records.length === 0) {
    return (
      <p className="rounded-md border border-neutral-200 bg-white p-4 text-xs text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-500">
        {t('history.empty')}
      </p>
    );
  }
  return (
    <ul className="flex flex-col gap-1 overflow-auto rounded-md border border-neutral-200 bg-white p-1 dark:border-neutral-800 dark:bg-neutral-900">
      {records.map((r) => (
        <li key={r.id}>
          <button
            type="button"
            onClick={() => onSelect(r.id)}
            aria-pressed={selectedId === r.id}
            className={
              (selectedId === r.id
                ? 'border-brand bg-brand/5 '
                : 'border-transparent hover:border-neutral-200 hover:bg-neutral-50 dark:hover:border-neutral-700 dark:hover:bg-neutral-800/60 ') +
              'flex w-full flex-col gap-0.5 rounded-md border px-2 py-1.5 text-left'
            }
          >
            <div className="flex items-center justify-between gap-2">
              <span className="line-clamp-1 text-xs text-neutral-500 dark:text-neutral-500">
                {formatDate(r.createdAt)}
              </span>
              {r.isFavorite ? (
                <Star
                  className="size-3 shrink-0 fill-amber-400 text-amber-400"
                  aria-label={t('history.detail.favorite')}
                />
              ) : null}
            </div>
            <p className="line-clamp-1 text-xs text-neutral-800 dark:text-neutral-200">
              {r.sourceText}
            </p>
            <p className="line-clamp-1 text-xs text-neutral-500 dark:text-neutral-500">
              {r.translatedText}
            </p>
          </button>
        </li>
      ))}
    </ul>
  );
}

function DetailPane({
  record,
  onToggleFavorite,
  onSetTags,
  onDelete,
}: {
  record: TranslationRecord | null;
  onToggleFavorite: () => void;
  onSetTags: (tags: string[]) => void;
  onDelete: () => void;
}) {
  const [copiedField, setCopiedField] = useState<'source' | 'translated' | null>(null);

  useEffect(() => {
    if (!copiedField) return;
    const timer = window.setTimeout(() => setCopiedField(null), 1500);
    return () => window.clearTimeout(timer);
  }, [copiedField]);

  if (!record) {
    return (
      <section className="flex items-center justify-center rounded-md border border-neutral-200 bg-white text-xs text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-500">
        {t('history.detail.empty')}
      </section>
    );
  }

  const handleCopy = (kind: 'source' | 'translated') => async () => {
    const text = kind === 'source' ? record.sourceText : record.translatedText;
    if (!text) return;
    try {
      await copyText(text);
      setCopiedField(kind);
    } catch {
      // 클립보드 실패는 silent — 알림은 v1 에서 노출하지 않음.
    }
  };

  return (
    <section className="flex flex-col gap-3 overflow-auto rounded-md border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
      <header className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-500">
          <span>{formatDate(record.createdAt)}</span>
          <span>·</span>
          <span className="font-mono">{record.model}</span>
          <span>·</span>
          <span>{record.durationMs}ms</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={onToggleFavorite}
            aria-pressed={record.isFavorite}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <Star
              className={record.isFavorite ? 'size-3.5 fill-amber-400 text-amber-400' : 'size-3.5'}
              aria-hidden
            />
            {record.isFavorite ? t('history.detail.unfavorite') : t('history.detail.favorite')}
          </button>
          <button
            type="button"
            onClick={onDelete}
            className="inline-flex items-center gap-1 rounded-md border border-rose-200 bg-white px-2 py-1 text-xs text-rose-700 hover:border-rose-300 hover:bg-rose-50 dark:border-rose-900 dark:bg-neutral-900 dark:text-rose-300 dark:hover:bg-rose-950"
          >
            <Trash2 className="size-3.5" aria-hidden />
            {t('history.detail.delete')}
          </button>
        </div>
      </header>

      <section className="flex flex-col gap-1">
        <div className="flex items-center justify-between">
          <h2 className="text-[10px] font-medium uppercase tracking-wider text-neutral-500 dark:text-neutral-500">
            {t('history.detail.source')}
          </h2>
          <button
            type="button"
            onClick={handleCopy('source')}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-1.5 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <Copy className="size-3" aria-hidden />
            {copiedField === 'source'
              ? t('translation.output.copied')
              : t('translation.output.copy')}
          </button>
        </div>
        <p className="whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50 p-3 text-sm leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100">
          {record.sourceText}
        </p>
      </section>

      <section className="flex flex-col gap-1">
        <div className="flex items-center justify-between">
          <h2 className="text-[10px] font-medium uppercase tracking-wider text-neutral-500 dark:text-neutral-500">
            {t('history.detail.translated')}
          </h2>
          <button
            type="button"
            onClick={handleCopy('translated')}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-1.5 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <Copy className="size-3" aria-hidden />
            {copiedField === 'translated'
              ? t('translation.output.copied')
              : t('translation.output.copy')}
          </button>
        </div>
        <p className="whitespace-pre-wrap rounded-md border border-neutral-200 bg-neutral-50 p-3 text-sm leading-relaxed text-neutral-900 dark:border-neutral-800 dark:bg-neutral-900/60 dark:text-neutral-100">
          {record.translatedText}
        </p>
      </section>

      <TagEditor tags={record.tags} onSubmit={onSetTags} />
    </section>
  );
}

function TagEditor({ tags, onSubmit }: { tags: string[]; onSubmit: (tags: string[]) => void }) {
  const [draft, setDraft] = useState('');

  const addTag = useCallback(() => {
    const t = draft.trim();
    if (!t) return;
    if (tags.some((existing) => existing === t)) {
      setDraft('');
      return;
    }
    onSubmit([...tags, t]);
    setDraft('');
  }, [draft, onSubmit, tags]);

  const removeTag = (tag: string) => onSubmit(tags.filter((existing) => existing !== tag));

  return (
    <section className="flex flex-col gap-1">
      <h2 className="text-[10px] font-medium uppercase tracking-wider text-neutral-500 dark:text-neutral-500">
        {t('history.detail.tags')}
      </h2>
      <div className="flex flex-wrap items-center gap-1">
        {tags.map((tag) => (
          <span
            key={tag}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-1.5 py-0.5 text-xs text-neutral-700 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300"
          >
            {tag}
            <button
              type="button"
              onClick={() => removeTag(tag)}
              aria-label={t('history.detail.tagRemove')}
              className="text-neutral-400 hover:text-rose-600 dark:hover:text-rose-400"
            >
              <X className="size-3" aria-hidden />
            </button>
          </span>
        ))}
        <input
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              addTag();
            }
          }}
          placeholder={t('history.detail.tagAddPlaceholder')}
          aria-label={t('history.detail.tagAddPlaceholder')}
          className="w-32 rounded-md border border-dashed border-neutral-300 bg-white px-2 py-0.5 text-xs text-neutral-900 placeholder:text-neutral-400 focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-600"
        />
        {draft.trim() ? (
          <button
            type="button"
            onClick={addTag}
            className="inline-flex items-center gap-1 rounded-md border border-neutral-300 bg-white px-1.5 py-0.5 text-[10px] text-neutral-700 hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800"
          >
            <Download className="size-3" aria-hidden />
            {t('history.detail.tagAdd')}
          </button>
        ) : null}
      </div>
    </section>
  );
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString('ko-KR', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}
