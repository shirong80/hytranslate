# 웹뷰 · CSP · 프론트엔드 보안 참조 가이드

웹뷰가 곧 네이티브 권한의 입구이므로, 프론트엔드 XSS는 일반 웹앱보다 영향이 훨씬 크다. CSP·dangerous 플래그·살균 누락을 점검한다. 설정은 `src-tauri/tauri.conf.json`의 `app.security`에 있다.

---

## 1. CSP 엄격성 (tauri.conf.json)

### 취약 패턴
```json
// app.security.csp — ❌ XSS 방어선이 사실상 무너짐
{ "csp": "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'" }
{ "csp": null }   // CSP 미설정
```
`unsafe-inline`은 인라인 스크립트/이벤트 핸들러 주입을, `unsafe-eval`은 `eval`/`new Function` 기반 페이로드 실행을 허용한다. 둘 다 XSS 차단을 무력화한다.

### 안전 패턴
```json
{
  "csp": "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' https://api.myservice.com"
}
```
`unsafe-inline`/`unsafe-eval`이 들어 있으면 반드시 이유를 따진다. 스타일에 한해 nonce/hash로 대체할 수 있는지 검토한다. `connect-src`로 IPC 외 네트워크 도착지도 제한한다.

---

## 2. dangerous 접두 플래그

다음 플래그가 켜져 있으면 강하게 의심한다.

| 플래그 | 위험 |
|--------|------|
| `app.security.dangerousDisableAssetCspModification` | Tauri가 자동 주입하는 asset CSP 보강을 끔 → CSP 우회 가능 |
| `assetProtocol.enable` + 넓은 `scope` | `asset:` 프로토콜로 임의 로컬 파일 로드 |
| `app.windows[].url`이 원격 URL | 원격 콘텐츠를 신뢰 웹뷰에 직접 로드 |

```json
// ❌ asset CSP 보강 비활성 + 광범위 asset scope
{ "security": { "dangerousDisableAssetCspModification": true, "assetProtocol": { "enable": true, "scope": ["$HOME/**"] } } }
```
`dangerous`로 시작하는 플래그는 기본적으로 켜지 않아야 한다. 켜져 있으면 정당 사유를 요구한다.

---

## 3. withGlobalTauri — 공격 표면 확대

### 취약 패턴
```json
// ❌ window.__TAURI__ 전역으로 API 노출 → XSS가 곧장 IPC 호출
{ "app": { "withGlobalTauri": true } }
```
전역 노출은 주입된 스크립트가 import 없이 바로 `window.__TAURI__.core.invoke(...)`를 부를 수 있게 해 공격 표면을 넓힌다.

### 안전 패턴
- 꼭 필요한 경우(번들러 미사용 등)가 아니면 `false`(기본값)로 두고, 프론트엔드에서 `@tauri-apps/api`를 모듈로 import한다.
- 켜야 한다면 capability/scope를 더 엄격히 가져간다.

---

## 4. Isolation Pattern

민감한 IPC를 다루는 앱은 **isolation pattern** 적용 여부를 검토한다. 이 패턴은 메인 웹뷰와 Tauri 코어 사이에 격리된 보조 웹뷰(IFrame)를 두어, IPC 메시지를 코어로 보내기 전에 가로채 검증·서명하게 한다.

```jsonc
// tauri.conf.json — isolation 적용 예
{ "app": { "security": { "pattern": { "use": "isolation", "options": { "dir": "../dist-isolation" } } } } }
```
XSS가 발생해도 isolation 레이어가 커맨드 호출을 검증하므로 권한 상승을 한 단계 더 막는다. 고위험 앱에서 미적용이면 P1로 권고한다.

---

## 5. 프론트엔드 XSS → 권한 상승

### 취약 패턴
```ts
// ❌ 비살균 동적 HTML 삽입 — Tauri에서는 곧 네이티브 권한 상승 경로
el.innerHTML = userProvidedMarkup;
```
```tsx
// ❌ React
<div dangerouslySetInnerHTML={{ __html: userContent }} />
```
일반 웹에서도 XSS지만, Tauri에서는 주입된 스크립트가 `invoke()`로 커맨드를 호출해 파일 접근·셸 실행으로 이어질 수 있다.

### 안전 패턴
```ts
// ✅ 텍스트로 삽입하거나 DOMPurify 등으로 살균
el.textContent = userProvidedText;
// 또는
el.innerHTML = DOMPurify.sanitize(userProvidedMarkup);
```
`innerHTML`/`outerHTML`/`insertAdjacentHTML`/`dangerouslySetInnerHTML`/프레임워크의 raw HTML 디렉티브에 사용자/원격 데이터가 살균 없이 들어가는지 본다. 가능하면 텍스트 삽입으로 대체한다.

---

## 6. 비밀·민감 로직의 위치

### 취약 패턴
```ts
// ❌ API 키/토큰을 프론트엔드 번들에 하드코딩 — 번들에서 추출 가능
const API_KEY = "sk-live-abcd1234";
```
프론트엔드 번들은 사용자가 열어볼 수 있다. 비밀키·토큰·서명 로직을 넣으면 안 된다.

### 안전 패턴
- 비밀키는 Rust(코어)에 두고, 프론트엔드는 결과만 IPC로 받는다.
- 민감 비즈니스 로직(라이선스 검증, 결제 서명 등)은 가능한 한 Rust 쪽으로 옮긴다.

---

## 7. 외부 링크 · 원격 콘텐츠

### 취약 패턴
```tsx
// ❌ 외부 링크가 앱 웹뷰에서 그대로 열림 — 신뢰 컨텍스트 오염
<a href="https://external.site">열기</a>   // target 처리 없음
```
```rust
// ❌ 외부 입력 URL을 검증 없이 셸 open
shell.open(user_url, None)
```

### 안전 패턴
- 외부 링크는 **시스템 브라우저**로 열리도록 처리한다(`@tauri-apps/plugin-opener` 또는 shell `open`을 화이트리스트된 http/https에 한정).
- `javascript:`, `file:`, `data:` 같은 위험 스킴을 차단한다.
- 원격 콘텐츠를 신뢰 웹뷰에 직접 로드하는 패턴(원격 URL을 윈도우 `url`로 지정)은 특별히 위험하므로 사유를 검토하고, 불가피하면 별도 저신뢰 윈도우 + 분리된 capability로 격리한다(→ `capabilities-scopes.md` 6).

---

## 리뷰 체크 요약
- [ ] CSP에 unsafe-inline/unsafe-eval이 없는가 (있으면 사유)
- [ ] CSP가 null/미설정이 아닌가
- [ ] dangerous* 플래그가 꺼져 있는가 (특히 dangerousDisableAssetCspModification)
- [ ] assetProtocol scope가 좁은가
- [ ] withGlobalTauri가 불필요하게 켜져 있지 않은가
- [ ] 고위험 IPC에 isolation pattern을 검토했는가
- [ ] innerHTML/dangerouslySetInnerHTML에 살균이 있는가
- [ ] 비밀키·토큰이 프론트엔드 번들에 없는가
- [ ] 외부 링크가 시스템 브라우저로 열리고 위험 스킴이 차단되는가
- [ ] 원격 콘텐츠를 신뢰 웹뷰에 직접 로드하지 않는가
