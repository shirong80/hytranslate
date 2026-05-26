export const ko = Object.freeze({
  'app.title': 'HyTranslate Mac',

  'translation.input.placeholder': '한국어, 중국어 간체, 중국어 번체 텍스트를 입력하세요.',
  'translation.input.charCount': '{count} / {limit}자',
  'translation.input.tooLong': '현재 화면에서는 최대 {limit}자까지 번역할 수 있습니다.',

  'translation.output.placeholder': '번역 결과가 여기에 표시됩니다.',
  'translation.output.copy': '복사',
  'translation.output.copied': '복사됨',
  'translation.output.retranslate': '다시 번역',

  'translation.status.idle': '대기 중',
  'translation.status.debouncing': '입력 중',
  'translation.status.translating': '번역 중…',
  'translation.status.completed': '완료',
  'translation.status.cancelled': '취소됨',
  'translation.status.error': '오류',
  'translation.status.duration': '{ms}ms',

  'translation.sourceLanguage.label': '입력 언어',
  'translation.sourceLanguage.auto': '자동 감지',
  'translation.sourceLanguage.korean': '한국어',
  'translation.sourceLanguage.chineseSimplified': '중국어 (간체)',
  'translation.sourceLanguage.chineseTraditional': '중국어 (번체)',
  'translation.sourceLanguage.detected.korean': '자동: 한국어',
  'translation.sourceLanguage.detected.chineseSimplified': '자동: 중국어 (간체)',
  'translation.sourceLanguage.detected.chineseTraditional': '자동: 중국어 (번체)',
  'translation.sourceLanguage.detected.unknown': '자동: 결정 안 됨',

  'translation.model.label': '모델',

  'nav.translate': '번역',
  'nav.settings': '설정',
  'nav.back': '뒤로',

  'settings.title': '설정',
  'settings.section.translation': '번역',
  'settings.section.appearance': '모양',
  'settings.activeModel.label': '활성 모델',
  'settings.activeModel.hy7b': 'Hy-MT2 7B (권장)',
  'settings.activeModel.hy1_8b': 'Hy-MT2 1.8B (저메모리)',
  'settings.ollamaEndpoint.label': 'Ollama endpoint',
  'settings.ollamaEndpoint.help':
    'localhost / 127.0.0.1 만 허용됩니다. 외부 호스트는 저장되지 않습니다.',
  'settings.theme.label': '테마',
  'settings.theme.system': '시스템',
  'settings.theme.light': '라이트',
  'settings.theme.dark': '다크',
  'settings.action.save': '저장',
  'settings.action.saving': '저장 중…',
  'settings.action.saved': '저장됨',

  'errors.OllamaUnavailable':
    'Ollama가 설치되어 있지 않습니다. HyTranslate Mac은 로컬 번역을 위해 Ollama가 필요합니다.',
  'errors.OllamaNotRunning':
    'Ollama가 실행 중이 아닙니다. 자동 실행을 시도하거나 직접 실행해 주세요.',
  'errors.ModelMissing': '선택한 번역 모델이 아직 다운로드되지 않았습니다.',
  'errors.InputTooLong': '현재 화면에서는 최대 {limit}자까지 번역할 수 있습니다.',
  'errors.Cancelled': '번역이 취소되었습니다.',
  'errors.NetworkBlocked': '네트워크 접근이 차단되었습니다.',
  'errors.Internal': '번역 중 문제가 발생했습니다. Ollama 상태를 확인한 뒤 다시 시도해 주세요.',
  'errors.action.retry': '다시 시도',
  'errors.action.openOllamaDownload': 'Ollama 공식 다운로드',
} as const);

export type I18nKey = keyof typeof ko;

export function t(key: I18nKey, params?: Record<string, string | number>): string {
  const template: string = ko[key] ?? key;
  if (!params) return template;
  return Object.entries(params).reduce<string>(
    (acc, [name, value]) => acc.replaceAll(`{${name}}`, String(value)),
    template,
  );
}
