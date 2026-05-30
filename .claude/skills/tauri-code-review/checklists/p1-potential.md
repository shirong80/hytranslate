# P1 — 잠재적 결함 체크리스트

당장 사고는 아니어도 특정 조건에서 문제가 되는 항목.

## async / 런타임 (references/ipc-command-security.md, rust-code-quality.md)
- [ ] async 커맨드 내 블로킹 I/O·CPU 작업이 spawn_blocking 처리됐는가
- [ ] async 구간을 가로지르는 std::sync::Mutex가 없는가 (필요 시 tokio Mutex)

## 에러 누출 (references/ipc-command-security.md, rust-code-quality.md)
- [ ] Result 에러 메시지에 내부 경로·스택·민감 정보가 없는가
- [ ] 커맨드 에러 타입이 Serialize를 구현하고 외부 노출 메시지가 일반화됐는가

## 공격 표면 (references/webview-csp-security.md)
- [ ] withGlobalTauri가 불필요하게 켜져 있지 않은가
- [ ] assetProtocol scope가 좁은가
- [ ] 고위험 IPC에 isolation pattern 적용을 검토했는가
- [ ] 외부 링크가 시스템 브라우저로 열리고 위험 스킴(javascript:/file:/data:)이 차단되는가
- [ ] 원격 콘텐츠를 신뢰 웹뷰에 직접 로드하지 않는가

## 윈도우 권한 분리 (references/capabilities-scopes.md)
- [ ] 권한 낮은 윈도우가 높은 권한 커맨드에 접근하지 못하도록 capability가 분리됐는가

## 타입 계약 (references/type-contracts-and-platform.md)
- [ ] serde rename 규약과 프론트엔드 필드명이 일치하는가
- [ ] Option/enum 직렬화 표현이 프론트 타입과 맞는가
- [ ] tauri-specta 등 자동 생성 바인딩이 최신으로 동기화·커밋됐는가
- [ ] 이벤트 emit/listen 이름·페이로드 타입이 양쪽에서 일치하는가

## 크로스플랫폼 (references/type-contracts-and-platform.md)
- [ ] 플랫폼 의존 코드의 #[cfg(...)] 분기가 누락 없이 처리됐는가
- [ ] 모바일 타깃이면 필요한 네이티브 권한이 매니페스트에 선언됐는가
- [ ] 플랫폼별 경로/파일시스템/웹뷰 엔진 차이가 반영됐는가

## 코드 안전 / 의존성 (references/rust-code-quality.md)
- [ ] unsafe 블록에 // SAFETY 주석과 실제 불변식 보장이 있는가
- [ ] Mutex/RwLock 포이즌 가능성을 처리하는가
- [ ] cargo audit 취약점이 없고 Cargo.lock이 커밋됐는가

## 회귀 방지 — 기존 정상 동작 보호 (references/type-contracts-and-platform.md)
이 변경이 "기존에 잘 되던 기능"을 깨뜨릴 수 있는데 그것을 막는 테스트가 있는지 본다. 특히 컴파일러가 잡아주지 못하는 IPC 경계 변경에 집중한다.
- [ ] 커맨드 시그니처(인자명/타입/추가·삭제) 변경 시, 변경하지 않은 기존 프론트엔드 호출부가 깨지지 않는가 (또는 함께 수정됐는가)
- [ ] serde rename/필드 추가·삭제로 기존 페이로드 역직렬화가 조용히 깨지지 않는가
- [ ] 이벤트 이름·페이로드 변경 시 기존 listen 측이 함께 갱신됐는가
- [ ] capability/scope를 축소·제거하면서 그 권한에 의존하던 기존 기능이 깨지지 않는가
- [ ] 공유 State·락 구조·전역 설정 변경이 기존 흐름의 동작을 바꾸지 않는가
- [ ] 위 변경 지점에 회귀 방지 테스트(기존 동작을 고정하는 단위/통합 테스트)가 있는가, 없다면 추가가 필요한가
- [ ] 기존 동작 변경이 의도된 것이라면 계획문서에 근거가 있는가 (의도치 않은 회귀 vs 의도된 변경 구분)
