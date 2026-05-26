import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { withExponentialBackoff, type CancellationSignal } from './backoff';

describe('withExponentialBackoff', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns immediately when isDone is true on the first call', async () => {
    const signal: CancellationSignal = { cancelled: false };
    const fn = vi.fn().mockResolvedValue('ready');
    const promise = withExponentialBackoff(fn, (v) => v === 'ready', signal);
    await vi.runAllTimersAsync();
    expect(await promise).toBe('ready');
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('retries on PRD §8.1 delay sequence until isDone', async () => {
    const signal: CancellationSignal = { cancelled: false };
    const fn = vi
      .fn()
      .mockResolvedValueOnce({ running: false })
      .mockResolvedValueOnce({ running: false })
      .mockResolvedValue({ running: true });

    const promise = withExponentialBackoff(fn, (v: { running: boolean }) => v.running, signal);
    await vi.runAllTimersAsync();
    const result = await promise;
    expect(result.running).toBe(true);
    expect(fn).toHaveBeenCalledTimes(3);
  });

  it('aborts sleep when signal.cancelled flips mid-wait', async () => {
    const signal: CancellationSignal = { cancelled: false };
    const fn = vi.fn().mockResolvedValue({ running: false });
    const promise = withExponentialBackoff(fn, (v: { running: boolean }) => v.running, signal);
    // 첫 fn() 호출이 끝난 뒤 cancel.
    await Promise.resolve();
    signal.cancelled = true;
    await vi.runAllTimersAsync();
    const result = await promise;
    expect(result.running).toBe(false);
    // cancel 이후에는 fn 이 추가로 호출되면 안 된다.
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('returns last value when delays exhaust without isDone', async () => {
    const signal: CancellationSignal = { cancelled: false };
    const fn = vi.fn().mockResolvedValue({ running: false });
    const promise = withExponentialBackoff(
      fn,
      (v: { running: boolean }) => v.running,
      signal,
      [10, 20],
    );
    await vi.runAllTimersAsync();
    const result = await promise;
    expect(result.running).toBe(false);
    expect(fn).toHaveBeenCalledTimes(3);
  });
});
