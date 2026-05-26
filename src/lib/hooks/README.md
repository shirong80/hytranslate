# lib/hooks

여러 feature 가 공유하는 cross-cutting React 훅.

- 단일 feature 에서만 쓰는 훅은 `features/<feature>/hooks/` 에 둔다.
- `useXxx` 네이밍, top-level 호출, cleanup 필수 (`useEffect` 구독은 반환 함수에서 해제).
