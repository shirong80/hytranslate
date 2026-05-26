// Rust `crate::settings::Settings` 와 1:1 mirror. variant 변경 시 양쪽 동시.

export const THEMES = ['System', 'Light', 'Dark'] as const;
export type Theme = (typeof THEMES)[number];

export interface Settings {
  globalHotkey: string;
  activeModel: string;
  autoCopyAfterTranslation: boolean;
  saveHistory: boolean;
  startAtLogin: boolean;
  hideDockIcon: boolean;
  ollamaEndpoint: string;
  theme: Theme;
  onboardingCompleted: boolean;
}

// PRD §9.2 기본값. 백엔드 `Settings::default()` 와 1:1 동기화.
// 백엔드 응답이 도착하기 전 FE 가 UI 를 그리는 동안의 placeholder 로 사용.
export const DEFAULT_SETTINGS: Settings = {
  globalHotkey: 'Cmd+Shift+T',
  activeModel: 'hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M',
  autoCopyAfterTranslation: false,
  saveHistory: true,
  startAtLogin: false,
  hideDockIcon: false,
  ollamaEndpoint: 'http://localhost:11434',
  theme: 'System',
  onboardingCompleted: false,
};
