export interface CancellationSignal {
  cancelled: boolean;
}

/**
 * 코드리뷰 Major 5 — PRD §8.1 의 reconnect 정책에 맞춰 250/500/1000/2000/4000/8000ms.
 * `isDone(value)` 가 truthy 일 때 즉시 종료. 끝까지 미해결이면 마지막 값 반환.
 * `signal.cancelled` 가 true 가 되면 대기와 호출을 즉시 종단.
 */
export async function withExponentialBackoff<T>(
  fn: () => Promise<T>,
  isDone: (v: T) => boolean,
  signal: CancellationSignal,
  delays: readonly number[] = [250, 500, 1000, 2000, 4000, 8000],
): Promise<T> {
  let last = await fn();
  if (signal.cancelled || isDone(last)) return last;
  for (const ms of delays) {
    await sleep(ms, signal);
    if (signal.cancelled) return last;
    last = await fn();
    if (isDone(last)) return last;
  }
  return last;
}

/**
 * `ms` 가 끝나거나 `signal.cancelled` 가 true 가 되는 즉시 resolve. fake-timer 환경에서도
 * 동작하도록 짧은 tick(50ms 또는 ms/4 중 작은 값)으로 폴링한다.
 */
function sleep(ms: number, signal: CancellationSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.cancelled) {
      resolve();
      return;
    }
    const startedAt = Date.now();
    const tickMs = Math.max(20, Math.min(50, Math.floor(ms / 4)));
    const tick = () => {
      if (signal.cancelled) {
        resolve();
        return;
      }
      if (Date.now() - startedAt >= ms) {
        resolve();
        return;
      }
      setTimeout(tick, tickMs);
    };
    setTimeout(tick, tickMs);
  });
}
