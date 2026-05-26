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
  'translation.status.typing': '입력 중',
  'translation.status.detecting': '언어 감지 중…',
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
  'settings.section.data': '데이터',
  'settings.legacyMigration.banner':
    '이전 위치({legacyDir})에 데이터 사본이 남아 있습니다. 새 위치로 정상 이전된 것을 확인했다면 정리할 수 있습니다.',
  'settings.legacyMigration.cleanup': '이전 위치 정리하기',
  'settings.legacyMigration.confirmTitle': '이전 위치 정리',
  'settings.legacyMigration.confirm':
    '이전 위치의 데이터를 새 위치의 legacy-backup-<timestamp>/ 폴더로 이동합니다. 현재 데이터에는 영향이 없습니다. 계속할까요?',
  'settings.legacyMigration.confirmPhraseLabel':
    '계속하려면 폴더 이름 "{phrase}"을(를) 정확히 입력하세요.',
  'settings.legacyMigration.confirmPhrasePlaceholder': '폴더 이름 입력',
  'settings.legacyMigration.confirmPhraseMismatch':
    '입력한 폴더 이름이 일치하지 않습니다. 정확히 "{phrase}"을(를) 입력해 주세요.',
  'settings.legacyMigration.cleanupBusy': '정리하는 중…',
  'settings.legacyMigration.completed':
    '완료했습니다. 이전 위치의 데이터를 {backupDir}에 백업했습니다. (이동된 파일: {moved}개)',
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
  'history.export.notice': '내보낸 파일에는 원문/결과가 평문으로 포함됩니다.',
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
  'errors.ClipboardEmpty': '클립보드에 텍스트가 없습니다.',
  'errors.ClipboardUnsupported': '이미지는 번역할 수 없습니다. 텍스트를 복사해 다시 시도해 주세요.',
  'errors.ClipboardReadFailed':
    '클립보드를 읽을 수 없습니다. macOS 손쉬운 사용 권한과 앱 권한을 확인해 주세요.',
  'errors.CopyFailed': '결과를 복사하지 못했습니다. 다시 시도해 주세요.',
  'errors.Internal': '번역 중 문제가 발생했습니다. Ollama 상태를 확인한 뒤 다시 시도해 주세요.',
  'errors.action.retry': '다시 시도',
  'errors.action.openOllamaDownload': 'Ollama 공식 다운로드',
  'errors.action.openSystemSettings': '시스템 설정 열기',

  'onboarding.title': '시작 준비',
  'onboarding.step.welcome': '환영',
  'onboarding.step.environment': '환경',
  'onboarding.step.ollama': 'Ollama',
  'onboarding.step.model': '모델',
  'onboarding.step.permissions': '권한',
  'onboarding.step.history': '이력',
  'onboarding.step.done': '완료',
  'onboarding.action.continue': '계속',
  'onboarding.action.back': '이전',
  'onboarding.action.recheck': '다시 확인',
  'onboarding.action.cancel': '취소',
  'onboarding.action.finish': '시작하기',
  'onboarding.action.finishing': '저장 중…',

  'onboarding.welcome.title': 'HyTranslate Mac에 오신 것을 환영합니다',
  'onboarding.welcome.description':
    'Tencent Hy-MT2 모델을 이 Mac에서 직접 실행해 한국어와 중국어를 영어로 번역합니다.',
  'onboarding.welcome.start': '시작',
  'onboarding.welcome.bullet.local': '번역 중 원문은 이 Mac 밖으로 전송되지 않습니다.',
  'onboarding.welcome.bullet.offline': '모델 다운로드 이후 네트워크 없이 동작합니다.',
  'onboarding.welcome.bullet.history':
    '번역 이력은 이 Mac에만 저장되며, 언제든 끄거나 삭제할 수 있습니다.',

  'onboarding.environment.title': '환경 확인',
  'onboarding.environment.description': '시스템 사양을 확인해 추천 모델을 결정합니다.',
  'onboarding.environment.checking': '환경을 확인하는 중…',
  'onboarding.environment.macos': 'macOS 버전',
  'onboarding.environment.macosUnsupported': 'macOS 13 이상이 필요합니다.',
  'onboarding.environment.arch': '아키텍처',
  'onboarding.environment.arch.appleSilicon': 'Apple Silicon',
  'onboarding.environment.arch.intel': 'Intel',
  'onboarding.environment.arch.unknown': '알 수 없음',
  'onboarding.environment.intelWarning': 'Intel Mac에서는 번역 속도가 느릴 수 있습니다.',
  'onboarding.environment.memory': '메모리',
  'onboarding.environment.lowMemory': '12 GB 미만 — 1.8B 모델을 권장합니다.',

  'onboarding.ollama.title': 'Ollama 준비',
  'onboarding.ollama.description': 'Ollama가 로컬에서 실행 중인지 확인합니다.',
  'onboarding.ollama.running': 'Ollama가 실행 중입니다.',
  'onboarding.ollama.installedButStopped':
    'Ollama가 설치되어 있지만 실행되지 않았습니다. 실행을 시도할 수 있습니다.',
  'onboarding.ollama.notInstalled':
    'Ollama가 설치되어 있지 않습니다. 공식 다운로드 후 실행해 주세요.',
  'onboarding.ollama.openDownload': '공식 다운로드 열기',
  'onboarding.ollama.tryStart': 'Ollama 실행 시도',

  'onboarding.model.title': '모델 다운로드',
  'onboarding.model.description': '추천 모델을 그대로 진행하거나 다른 모델을 선택할 수 있습니다.',
  'onboarding.model.choose': '모델 선택',
  'onboarding.model.recommended': '추천',
  'onboarding.model.installed': '설치됨',
  'onboarding.model.alreadyInstalled': '선택한 모델이 이미 설치되어 있습니다.',
  'onboarding.model.startPull': '선택한 모델 다운로드',
  'onboarding.model.pulling': '{model} 다운로드 중…',
  'onboarding.model.hy7b.label': 'Hy-MT2 7B',
  'onboarding.model.hy7b.sub': '약 4 GB · 권장 사양 12 GB RAM 이상',
  'onboarding.model.hy1_8b.label': 'Hy-MT2 1.8B',
  'onboarding.model.hy1_8b.sub': '약 1 GB · 저메모리 환경용 (8 GB RAM 이상)',

  'onboarding.permissions.title': '권한 안내',
  'onboarding.permissions.description':
    '플로팅 팝업과 전역 단축키를 사용하려면 macOS 손쉬운 사용 권한이 필요합니다.',
  'onboarding.permissions.accessibility':
    'Cmd+Shift+T 등 전역 단축키 등록을 위해 시스템 설정 → 손쉬운 사용에서 HyTranslate Mac을 허용해 주세요.',
  'onboarding.permissions.note': '나중에 설정 화면의 안내에서 다시 요청할 수 있습니다.',
  'onboarding.permissions.openSettings': '시스템 설정 열기',

  'onboarding.history.title': '이력 저장 안내',
  'onboarding.history.description':
    '기본적으로 번역 이력은 이 Mac에만 저장됩니다. 언제든 설정에서 끄거나 전체 삭제할 수 있습니다.',
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
