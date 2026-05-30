# P0 — 치명적 체크리스트 (머지 전 반드시 해결)

프로덕션 보안 사고나 앱 전체 다운으로 직결되는 항목. 보안 두 축을 먼저 통과시킨다.

## 축① IPC 커맨드 입력 검증 (references/ipc-command-security.md)
- [ ] 모든 커맨드 인자를 신뢰 불가 입력으로 보고 검증하는가
- [ ] 경로 인자에 canonicalize + base 디렉터리 prefix 강제가 있는가 (path traversal)
- [ ] 절대경로/`..`/심볼릭 링크로 샌드박스를 벗어날 수 없는가
- [ ] 셸/프로세스 실행에 문자열 조립 대신 인자 분리 전달을 쓰는가 (command injection)
- [ ] SQL이 파라미터 바인딩을 쓰는가 (SQL injection)
- [ ] 범위·길이·enum 검증이 Rust 쪽에 있는가 (프론트 검증에만 의존 금지)
- [ ] 임의 read/write/exec 같은 만능 위험 프리미티브를 커맨드로 노출하지 않는가

## 축② 권한 · Capability · Scope (references/capabilities-scopes.md)
- [ ] default capability가 실제 쓰는 권한만 담는가 (불필요 권한 제거)
- [ ] `core:*`/플러그인 권한이 사용분으로 최소화됐는가
- [ ] `windows`가 한정적인가 (저신뢰 윈도우에 강한 권한 미부여)
- [ ] fs scope가 앱 데이터 디렉터리로 한정되는가 (`$HOME/**`·`**` 금지)
- [ ] shell scope의 cmd/args가 고정 또는 검증되는가 (`args: true` 경계)
- [ ] http scope가 필요한 도메인만 허용하는가 (https:// 전체·평문 http 경계)
- [ ] remote 필드가 외부 origin에 커맨드를 노출하지 않는가 (있으면 정당 사유 필수)

## CSP · 보안 설정 (references/webview-csp-security.md)
- [ ] CSP에 unsafe-inline/unsafe-eval이 없는가 (있으면 사유 확인)
- [ ] CSP가 null/미설정이 아닌가
- [ ] dangerousDisableAssetCspModification 등 dangerous* 플래그가 꺼져 있는가

## XSS → 권한 상승 (references/webview-csp-security.md)
- [ ] innerHTML/dangerouslySetInnerHTML/동적 HTML 삽입에 살균(sanitize)이 있는가
- [ ] 비밀키·토큰이 프론트엔드 번들에 하드코딩돼 있지 않은가

## 업데이터 · 코드 서명 (references/type-contracts-and-platform.md)
- [ ] 업데이터 엔드포인트가 모두 HTTPS인가
- [ ] minisign 서명 검증이 활성화되고 pubkey가 올바른가
- [ ] 서명용 private key가 저장소·CI 로그·diff에 노출되지 않았는가

## 패닉 · 동시성 (references/rust-code-quality.md)
- [ ] 사용자/웹뷰 입력 경로에 unwrap()/expect() 패닉이 없는가
- [ ] 콜백·핸들러가 보유 중인 락을 중첩 획득하는 데드락이 없는가
- [ ] 공유 State에 데이터 레이스가 없는가
