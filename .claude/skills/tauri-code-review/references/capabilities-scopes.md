# 권한 · Capability · Scope 참조 가이드 (축②)

Tauri 1의 allowlist가 Tauri 2에서 **capabilities 시스템**으로 바뀌었다. 리뷰 포인트가 가장 많은 영역이다. `src-tauri/capabilities/*.json`에서 최소 권한 원칙이 지켜지는지, scope가 과도하게 열려 있지 않은지를 본다. scope 설정이 곧 보안이다.

---

## 1. 최소 권한 원칙 — default capability 과다

### 취약 패턴
```json
// capabilities/default.json — ❌ 모든 윈도우에 광범위 권한을 한 번에 부여
{
  "identifier": "default",
  "windows": ["*"],
  "permissions": [
    "core:default",
    "fs:default",
    "shell:default",
    "http:default",
    "fs:allow-read-file",
    "fs:allow-write-file",
    "shell:allow-execute"
  ]
}
```
실제로 쓰지 않는 권한까지 default에 모아두면 공격 표면이 불필요하게 넓어진다.

### 안전 패턴
```json
// ✅ 실제 사용하는 권한만, 윈도우를 한정
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "core:event:default",
    "core:window:allow-set-title",
    { "identifier": "fs:allow-read-text-file", "allow": [{ "path": "$APPDATA/myapp/**" }] }
  ]
}
```
점검: ① `core:*`와 플러그인 권한이 **실제 쓰는 것만** 부여됐는가, ② `*-default` 묶음 권한이 필요 이상으로 많은 동작을 켜지 않는가, ③ `allow-execute`, `allow-write-file` 같은 강한 권한에 정당 사유가 있는가.

---

## 2. fs scope — 가장 위험한 과다 허용

### 취약 패턴
```json
// ❌ 홈 디렉터리 전체 — 사실상 권한 통제 무력화
{ "identifier": "fs:scope", "allow": [{ "path": "$HOME/**" }] }
{ "identifier": "fs:scope", "allow": [{ "path": "**" }] }
```

### 안전 패턴
```json
// ✅ 앱 데이터 디렉터리로 한정 + 민감 경로 deny
{
  "identifier": "fs:scope",
  "allow": [{ "path": "$APPDATA/myapp/**" }, { "path": "$APPCACHE/myapp/**" }],
  "deny": [{ "path": "$APPDATA/myapp/secrets/**" }]
}
```
fs 허용 경로는 `$APPDATA`/`$APPCACHE`/`$APPLOCALDATA` 등 앱 전용 디렉터리로 한정한다. `$HOME/**`, `$DOCUMENT/**` 전체, `**` 같은 광범위 와일드카드는 거부 대상이다. `deny`는 `allow`보다 우선 적용되므로 민감 하위 경로를 명시 차단할 수 있다.

---

## 3. shell scope — 실행 가능 명령 통제

### 취약 패턴
```json
// ❌ 임의 명령/인자 실행 허용
{ "identifier": "shell:allow-execute", "allow": [{ "name": "sh", "cmd": "sh", "args": true }] }
```
`"args": true`는 임의 인자를 허용한다 → 사실상 임의 명령 실행.

### 안전 패턴
```json
// ✅ 명령과 인자 형식을 고정
{
  "identifier": "shell:allow-execute",
  "allow": [{
    "name": "git-status",
    "cmd": "git",
    "args": ["status", "--porcelain"]
  }]
}
```
또는 정규식으로 인자 형식을 제약: `"args": [{ "validator": "\\d+" }]`. sidecar 사용 시에도 인자가 사용자 입력으로 조립되지 않는지 확인한다(→ 명령어 인젝션, `ipc-command-security.md` 2).

---

## 4. http scope — 허용 URL 범위

### 취약 패턴
```json
// ❌ 모든 origin 허용 → SSRF·임의 외부 요청 통로
{ "identifier": "http:default", "allow": [{ "url": "https://**" }] }
{ "identifier": "http:default", "allow": [{ "url": "http://*" }] }
```

### 안전 패턴
```json
// ✅ 필요한 도메인만
{ "identifier": "http:default", "allow": [{ "url": "https://api.myservice.com/*" }] }
```
와일드카드로 전체 인터넷을 여는지, `http://`(평문)를 허용하는지 본다. 웹뷰가 임의 URL로 요청을 보낼 수 있으면 내부망 SSRF·데이터 유출 통로가 된다.

---

## 5. remote 필드 — 외부 origin에 커맨드 노출 (매우 위험)

### 취약 패턴
```json
// ❌ 외부 웹사이트에서 우리 앱 커맨드를 호출하도록 허용
{
  "identifier": "main",
  "windows": ["main"],
  "remote": { "urls": ["https://*.example.com"] },
  "permissions": ["fs:allow-read-file", "shell:allow-execute"]
}
```
`remote`는 원격 origin(웹뷰에 로드된 외부 페이지)이 이 capability의 커맨드를 호출하도록 허용한다. 외부 사이트가 탈취되거나 MITM되면 그대로 네이티브 권한이 넘어간다.

### 안전 패턴
- 정당한 사유(신뢰된 자사 도메인 + 명확한 필요)가 없으면 `remote` 사용을 **거부 대상**으로 본다.
- 불가피하면 노출 권한을 최소(읽기 전용·비위험)로 줄이고, fs/shell/http 같은 강한 권한은 remote capability에서 제외한다.

---

## 6. 윈도우별 capability 분리

여러 윈도우가 있을 때, 권한이 낮아야 할 윈도우(예: 외부 콘텐츠 표시용)가 높은 권한 커맨드에 접근하지 못하도록 capability를 윈도우별로 분리한다.

### 취약 패턴
```json
// ❌ 외부 콘텐츠를 띄우는 윈도우까지 강한 권한 capability에 포함
{ "identifier": "admin", "windows": ["*"], "permissions": ["fs:allow-write-file", "shell:allow-execute"] }
```

### 안전 패턴
```json
// ✅ 신뢰 윈도우와 외부/저신뢰 윈도우의 권한을 분리
{ "identifier": "main-trusted", "windows": ["main"], "permissions": ["fs:allow-write-file"] }
{ "identifier": "external-view", "windows": ["external"], "permissions": ["core:event:allow-listen"] }
```
`windows`(및 모바일의 `webviews`) 매칭이 의도한 윈도우에만 적용되는지, 와일드카드(`*`)가 저신뢰 윈도우까지 끌어들이지 않는지 확인한다.

---

## 7. 플러그인 권한 일반 원칙

`sql`, `store`, `updater`, `dialog`, `notification`, `os`, `clipboard` 등 다른 플러그인도 같은 기준으로 점검한다:
- 노출 범위가 실제 사용에 필요한 최소인가
- scope가 있는 플러그인(fs/shell/http/sql)은 scope가 좁게 설정됐는가
- `clipboard-manager` 읽기, `dialog` 임의 경로 등 민감 동작에 과한 권한이 없는가

---

## 리뷰 체크 요약
- [ ] default capability가 실제 쓰는 권한만 담는가 (불필요 권한 제거)
- [ ] `windows`가 한정적인가 (저신뢰 윈도우에 강한 권한 미부여)
- [ ] fs scope가 앱 데이터 디렉터리로 한정되는가 ($HOME/**·** 금지)
- [ ] shell scope의 cmd/args가 고정 또는 검증되는가 (`args: true` 경계)
- [ ] http scope가 필요한 도메인만 허용하는가 (https:// 전체·평문 경계)
- [ ] remote 필드가 있으면 정당 사유가 있는가 (없으면 거부)
- [ ] 강한 권한이 deny로 민감 경로를 명시 차단하는가
