const POPUP_MIN_HEIGHT = 360;
const POPUP_BASE_HEIGHT = 240;
const CHARS_PER_LINE = 64;
const LINE_HEIGHT = 18;
const MAX_HEIGHT_RATIO = 0.8;

// output 길이에 따라 popup 높이를 계산한다. monitor 논리 높이의 80% 를 cap, 360 이 하한.
export function computePopupHeight(outputLength: number, monitorLogicalHeight: number): number {
  const maxH = monitorLogicalHeight * MAX_HEIGHT_RATIO;
  const lines = outputLength > 0 ? Math.ceil(outputLength / CHARS_PER_LINE) : 0;
  const desired = POPUP_BASE_HEIGHT + lines * LINE_HEIGHT;
  return Math.min(Math.max(desired, POPUP_MIN_HEIGHT), maxH);
}

export type PopupResizer = (
  outputLength: number,
  monitorLogicalHeight: number,
  resize: (height: number) => Promise<void>,
  isCancelled: () => boolean,
) => Promise<void>;

// 높이 변경(setSize)+top-left 보존은 Rust resize_popup 가 메인스레드에서 동기 처리한다. 다만
// 여러 resize_popup invoke 가 동시에 in-flight 면 async command·main-thread task 의 enqueue 순서가
// 호출 순서와 달라져 최신이 아닌 높이가 마지막에 적용될 수 있다(Rust 에 seq state 를 두지 않기로
// 함). 그래서 FE 가 single-flight 로 직렬화한다: 한 번에 하나만 in-flight 로 보내고, 대기 중
// target 은 최신값 하나로 coalesce 한다. 직렬화로 enqueue 순서 = 호출 순서 → resize_popup 의
// FIFO 적용 → 최신 output 높이가 최종 적용된다.
export function createPopupResizer(): PopupResizer {
  let chain: Promise<void> = Promise.resolve();
  let pending: {
    height: number;
    resize: (height: number) => Promise<void>;
    isCancelled: () => boolean;
  } | null = null;

  return (outputLength, monitorLogicalHeight, resize, isCancelled) => {
    if (isCancelled()) return Promise.resolve();
    pending = {
      height: computePopupHeight(outputLength, monitorLogicalHeight),
      resize,
      isCancelled,
    };
    // 직전 resize 가 reject 돼도 catch 로 흡수해 chain 을 복구한다 — 안 그러면 한 번 실패한 뒤
    // 모든 후속 콜백이 rejected chain 의 .then 에 묶여 영구히 실행되지 않는다(P1-1).
    chain = chain
      .catch(() => undefined)
      .then(async () => {
        const next = pending;
        pending = null;
        // enqueue 후 effect cleanup 등으로 취소됐으면 invoke 직전에 다시 검사해 stale resize 를 막는다(P2-1).
        if (next && !next.isCancelled()) await next.resize(next.height);
      });
    return chain;
  };
}
