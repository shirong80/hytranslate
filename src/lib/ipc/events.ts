export const TRANSLATION_STARTED = 'translation:started' as const;
export const TRANSLATION_CHUNK = 'translation:chunk' as const;
export const TRANSLATION_COMPLETED = 'translation:completed' as const;
export const TRANSLATION_CANCELLED = 'translation:cancelled' as const;
export const TRANSLATION_ERROR = 'translation:error' as const;

export const MODEL_PULL_STARTED = 'model-pull:started' as const;
export const MODEL_PULL_PROGRESS = 'model-pull:progress' as const;
export const MODEL_PULL_COMPLETED = 'model-pull:completed' as const;
export const MODEL_PULL_ERROR = 'model-pull:error' as const;

export const POPUP_OPENED = 'popup:opened' as const;
export const POPUP_CLOSED = 'popup:closed' as const;
export const MENUBAR_OPENED = 'menubar:opened' as const;
export const MENUBAR_CLOSED = 'menubar:closed' as const;

export const NAV_REQUEST = 'nav:request' as const;

export type NavRoute = 'translate' | 'history' | 'settings';
export interface NavRequestPayload {
  route: NavRoute;
}

export type TranslationEvent =
  | typeof TRANSLATION_STARTED
  | typeof TRANSLATION_CHUNK
  | typeof TRANSLATION_COMPLETED
  | typeof TRANSLATION_CANCELLED
  | typeof TRANSLATION_ERROR;

export type ModelPullEvent =
  | typeof MODEL_PULL_STARTED
  | typeof MODEL_PULL_PROGRESS
  | typeof MODEL_PULL_COMPLETED
  | typeof MODEL_PULL_ERROR;

export type SurfaceEvent =
  | typeof POPUP_OPENED
  | typeof POPUP_CLOSED
  | typeof MENUBAR_OPENED
  | typeof MENUBAR_CLOSED
  | typeof NAV_REQUEST;

export type EventName = TranslationEvent | ModelPullEvent | SurfaceEvent;
