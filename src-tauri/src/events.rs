/// IPC 이벤트 이름 단일 정의.
/// FE 의 `src/lib/ipc/events.ts` 와 1:1 mirror. 양쪽 동시 변경.
pub const TRANSLATION_STARTED: &str = "translation:started";
pub const TRANSLATION_CHUNK: &str = "translation:chunk";
pub const TRANSLATION_COMPLETED: &str = "translation:completed";
pub const TRANSLATION_CANCELLED: &str = "translation:cancelled";
pub const TRANSLATION_ERROR: &str = "translation:error";

pub const MODEL_PULL_STARTED: &str = "model-pull:started";
pub const MODEL_PULL_PROGRESS: &str = "model-pull:progress";
pub const MODEL_PULL_COMPLETED: &str = "model-pull:completed";
pub const MODEL_PULL_ERROR: &str = "model-pull:error";
