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
  'nav.history': '이력',
  'nav.settings': '설정',
  'nav.back': '뒤로',

  'settings.title': '설정',
  'settings.section.translation': '번역',
  'settings.section.shortcut': '단축키',
  'settings.section.system': '시스템',
  'settings.section.appearance': '모양',
  'settings.activeModel.label': '활성 모델',
  'settings.activeModel.hy7b': 'Hy-MT2 7B (권장)',
  'settings.activeModel.hy1_8b': 'Hy-MT2 1.8B (저메모리)',
  'settings.ollamaEndpoint.label': 'Ollama endpoint',
  'settings.ollamaEndpoint.help':
    'localhost / 127.0.0.1 만 허용됩니다. 외부 호스트는 저장되지 않습니다.',
  'settings.globalHotkey.label': '플로팅 팝업 단축키',
  'settings.globalHotkey.help':
    '예시: Cmd+Shift+T. 변경 후 즉시 재등록됩니다. macOS 손쉬운 사용 권한이 필요합니다.',
  'settings.autoCopy.label': '번역 완료 시 결과 자동 복사',
  'settings.saveHistory.label': '번역 기록 저장',
  'settings.deleteAllHistory.label': '전체 번역 이력 삭제',
  'settings.startAtLogin.label': '로그인 시 자동 시작',
  'settings.hideDockIcon.label': 'Dock 아이콘 숨기기 (메뉴바 전용 모드)',
  'settings.theme.label': '테마',
  'settings.theme.system': '시스템',
  'settings.theme.light': '라이트',
  'settings.theme.dark': '다크',
  'settings.action.save': '저장',
  'settings.action.saving': '저장 중…',
  'settings.action.saved': '저장됨',

  'popup.title': '빠른 번역',
  'popup.input.placeholder': '번역할 텍스트를 입력하세요.',
  'popup.action.close': '닫기',
  'popup.action.copy': '복사',
  'popup.action.copied': '복사됨',
  'popup.shortcuts.hint': 'Cmd+Enter 번역 · Cmd+C 결과 복사 · Esc 닫기',

  'menubar.input.placeholder': '간단 번역…',
  'menubar.recent.title': '최근 5건',
  'menubar.recent.empty': '아직 번역 기록이 없습니다.',
  'menubar.action.copyClipboard': '클립보드 번역',

  'history.title': '번역 이력',
  'history.total': '{count}건',
  'history.loading': '불러오는 중…',
  'history.empty': '저장된 번역 이력이 없습니다.',
  'history.search.placeholder': '원문 또는 번역 결과 검색',
  'history.filter.tagPlaceholder': '태그 필터',
  'history.filter.favoriteOnly': '즐겨찾기만',
  'history.deleteAll': '전체 삭제',
  'history.deleteAll.confirm': '모든 번역 이력을 삭제할까요? 되돌릴 수 없습니다.',
  'history.export.csv': 'CSV 내보내기',
  'history.export.json': 'JSON 내보내기',
  'history.export.success': '{count}건을 {path}에 저장했습니다.',
  'history.detail.empty': '왼쪽 목록에서 항목을 선택하세요.',
  'history.detail.source': '원문',
  'history.detail.translated': '번역',
  'history.detail.favorite': '즐겨찾기',
  'history.detail.unfavorite': '즐겨찾기 해제',
  'history.detail.delete': '삭제',
  'history.detail.tags': '태그',
  'history.detail.tagRemove': '태그 제거',
  'history.detail.tagAdd': '추가',
  'history.detail.tagAddPlaceholder': '새 태그',

  'errors.OllamaUnavailable':
    'Ollama가 설치되어 있지 않습니다. HyTranslate Mac은 로컬 번역을 위해 Ollama가 필요합니다.',
  'errors.OllamaNotRunning':
    'Ollama가 실행 중이 아닙니다. 자동 실행을 시도하거나 직접 실행해 주세요.',
  'errors.ModelMissing': '선택한 번역 모델이 아직 다운로드되지 않았습니다.',
  'errors.InputTooLong': '현재 화면에서는 최대 {limit}자까지 번역할 수 있습니다.',
  'errors.Cancelled': '번역이 취소되었습니다.',
  'errors.NetworkBlocked': '네트워크 접근이 차단되었습니다.',
  'errors.PermissionRequired': '전역 단축키를 사용하려면 macOS 손쉬운 사용 권한 설정이 필요합니다.',
  'errors.InvalidShortcut': '단축키 형식이 올바르지 않습니다: {input}',
  'errors.Internal': '번역 중 문제가 발생했습니다. Ollama 상태를 확인한 뒤 다시 시도해 주세요.',
  'errors.action.retry': '다시 시도',
  'errors.action.openOllamaDownload': 'Ollama 공식 다운로드',
  'errors.action.openSystemSettings': '시스템 설정 열기',
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
