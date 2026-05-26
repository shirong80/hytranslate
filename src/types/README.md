# types

여러 feature 가 공유하는 공용 타입.

- 단일 feature 도메인 타입은 `features/<feature>/types.ts` 에 둔다.
- 브랜드 타입 (`type RequestId = string & { __brand: 'RequestId' }`) 처럼 cross-cutting ID 타입을 여기 둔다.
- IPC 페이로드 타입은 백엔드 serde 구조와 1:1 mirror.
