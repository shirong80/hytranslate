import { describe, expect, it, vi } from 'vitest';

import { computePopupHeight, createPopupResizer } from './resize';

describe('computePopupHeight', () => {
  it('returns the 360 floor for empty output', () => {
    expect(computePopupHeight(0, 1080)).toBe(360);
  });

  it('grows with output length up to the 80% monitor cap', () => {
    // 1000 chars → 16 lines → 240 + 16*18 = 528, under the 864 cap.
    expect(computePopupHeight(1000, 1080)).toBe(528);
  });

  it('caps at 80% of the monitor logical height', () => {
    // huge output would exceed the cap → clamped to 1080 * 0.8 = 864.
    expect(computePopupHeight(100000, 1080)).toBe(864);
  });

  it('restores a taller height when the same long output is re-measured on a larger screen (P2-1)', () => {
    // long output capped on a short screen, then re-measured on a tall screen → grows back.
    const onShort = computePopupHeight(100000, 900); // capped at 900 * 0.8 = 720
    const onTall = computePopupHeight(100000, 1600); // capped at 1600 * 0.8 = 1280
    expect(onShort).toBe(720);
    expect(onTall).toBe(1280);
    expect(onTall).toBeGreaterThan(onShort);
  });
});

interface Deferred {
  promise: Promise<void>;
  resolve: () => void;
}

function deferred(): Deferred {
  let resolve!: () => void;
  const promise = new Promise<void>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

describe('createPopupResizer', () => {
  it('applies the computed height once when not cancelled', async () => {
    const resize = vi.fn(async (_height: number) => {});
    const run = createPopupResizer();

    await run(1000, 1080, resize, () => false);

    expect(resize).toHaveBeenCalledOnce();
    expect(resize).toHaveBeenCalledWith(528);
  });

  it('skips when already cancelled (coalesce before invoke)', async () => {
    const resize = vi.fn(async (_height: number) => {});
    const run = createPopupResizer();

    await run(1000, 1080, resize, () => true);

    expect(resize).not.toHaveBeenCalled();
  });

  it('serializes overlapping calls so the latest height is applied last (latest-wins)', async () => {
    const applied: number[] = [];
    const gate = deferred();
    const entered = deferred();
    let first = true;
    const resize = vi.fn(async (height: number) => {
      applied.push(height);
      if (first) {
        first = false;
        entered.resolve(); // A 가 in-flight 에 진입했음을 알린다.
        await gate.promise; // 먼저 시작된 resize 를 늦게 완료시킨다.
      }
    });
    const run = createPopupResizer();

    // A 시작 → in-flight 진입 대기 → B 를 대기열에 넣고 A 를 늦게 완료시킨다.
    const a = run(0, 1080, resize, () => false); // height 360
    await entered.promise; // A 의 in-flight 단계 진입(resize(360) 호출 후 gate 대기)
    const b = run(100000, 1080, resize, () => false); // height 864 (최신)
    gate.resolve();
    await Promise.all([a, b]);

    // 한 번에 하나만 in-flight → 순서대로 적용되고 마지막은 최신(864).
    expect(applied).toEqual([360, 864]);
  });

  it('recovers the queue after a resize rejects so the next request still applies (P1-1)', async () => {
    const applied: number[] = [];
    let first = true;
    const resize = vi.fn(async (height: number) => {
      if (first) {
        first = false;
        throw new Error('transient resize failure');
      }
      applied.push(height);
    });
    const run = createPopupResizer();

    // 첫 요청은 reject 된다 — caller(applyResize)처럼 흡수한다.
    await run(0, 1080, resize, () => false).catch(() => undefined); // height 360, reject
    // 두 번째 요청은 복구된 chain 위에서 실제로 적용돼야 한다.
    await run(1000, 1080, resize, () => false); // height 528

    expect(applied).toEqual([528]);
  });

  it('does not invoke a pending resize that was cancelled while waiting in queue (P2-1)', async () => {
    const applied: number[] = [];
    const gate = deferred();
    const entered = deferred();
    let first = true;
    const resize = vi.fn(async (height: number) => {
      applied.push(height);
      if (first) {
        first = false;
        entered.resolve(); // A 가 in-flight 에 진입했음을 알린다.
        await gate.promise; // A 를 in-flight 로 붙잡아 둔다.
      }
    });
    const run = createPopupResizer();

    // A 시작 → in-flight 진입 → B 를 대기열에 넣고 B 만 취소한 뒤 A 를 완료시킨다.
    const a = run(0, 1080, resize, () => false); // height 360
    await entered.promise;
    let bCancelled = false;
    const b = run(1000, 1080, resize, () => bCancelled); // height 528 (대기)
    bCancelled = true; // enqueue 후 effect cleanup 등으로 취소.
    gate.resolve();
    await Promise.all([a, b]);

    // A 만 적용되고 취소된 B 는 invoke 되지 않는다.
    expect(applied).toEqual([360]);
  });
});
