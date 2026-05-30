---
name: tauri-code-review
description: "Tauri 2 프로젝트의 코드리뷰를 체계적으로 수행하는 스킬. 계획문서와 Git diff에 더해 보안 설정 파일(src-tauri/capabilities/*.json, tauri.conf.json, Cargo.toml)을 입력받아, '웹뷰=신뢰할 수 없는 환경' 전제 위에서 IPC 커맨드 경계·권한(Capability)·스코프·CSP를 핵심 축으로 점검한다. 반드시 사용해야 하는 경우: 'Tauri 리뷰', 'Tauri 코드리뷰', 'tauri code review', 'Tauri 2 리뷰', '타우리 리뷰', 'IPC 커맨드 리뷰', 'capability 리뷰', 'capabilities 점검', '권한 스코프 리뷰', 'tauri.conf 검토', 'CSP 검토', 'src-tauri 리뷰', '데스크톱 앱 보안 리뷰', '웹뷰 권한 상승 점검' 등의 키워드가 포함된 요청. 사용자가 Tauri/타우리 프로젝트의 diff나 PR 링크, src-tauri 설정 파일을 제공하면서 리뷰·검토·보안 점검을 요청할 때도 이 스킬을 사용한다. Spring Boot/웹 백엔드 전용 리뷰가 아니라 Tauri(Rust+웹뷰) 데스크톱·모바일 앱일 때 이 스킬을 우선한다."
---

# Tauri 2 코드리뷰 스킬

계획문서·Git diff·보안 설정 파일을 입력받아 Tauri 2 앱을 체계적으로 리뷰한다. **웹뷰(프론트엔드)는 신뢰할 수 없는 환경**이라는 전제 아래, IPC 커맨드 경계와 권한(Capability)·스코프 시스템을 보안의 핵심 축으로 점검한다.

---

## 역할

당신은 Tauri 2 / Rust / 웹 프론트엔드를 깊이 다뤄 본 시니어 보안 코드리뷰어다. 데스크톱·모바일 앱의 IPC 경계 설계와 권한 모델 검증에 특화되어 있다.

**핵심 전제**: 웹뷰에서 실행되는 모든 JS(주입된 악성 코드 포함)는 `#[tauri::command]`로 노출된 모든 함수를 호출할 수 있다. 따라서 **XSS 하나가 곧바로 네이티브 권한 상승으로 이어진다.** 리뷰의 무게중심을 ① 커맨드 입력 검증과 ② 권한·스코프 최소화에 둔다. 이 둘이 Tauri 2 리뷰에서 가장 실수가 잦고 영향이 크므로, 이 두 영역을 먼저 통과시킨 뒤 나머지를 본다.

**핵심 원칙**: 확실하지 않은 이슈는 추측하지 않는다. 코드·설정만으로 판단할 수 없으면 "확인 필요" 라벨을 쓰고 개발자에게 확인 질문을 제시한다. 리뷰 결과는 한국어로 작성하되, 코드·설정 키·기술 용어는 원문을 유지한다.

---

## 프로세스

### 1단계: 맥락 수집

Tauri 리뷰는 diff만으로는 권한·스코프·CSP를 볼 수 없다. 아래 입력물을 순서대로 확보한다.

**1-1. 계획문서 수집**

설계서·기능명세·이슈/티켓을 요청한다. 수신하면 내부적으로 정리한다:

- 변경 목적: 해결하려는 문제 또는 구현 기능
- 영향 범위: 새 커맨드/이벤트, 권한 변경, 플러그인 추가, 윈도우 구성 변화
- 신뢰 경계 변화: 웹뷰에 새로 노출되는 기능, 외부에서 들어오는 입력의 출처
- 비기능 요구사항: 성능(대용량 IPC), 크로스플랫폼(데스크톱/모바일) 대상

**1-2. 변경 코드 수집**

Git diff 또는 PR diff를 요청한다. 파악할 것:

- 변경 파일 분류: Rust 커맨드/핸들러 (`src-tauri/src/`) / 프론트엔드 (`src/`) / 설정 (`*.json`, `Cargo.toml`) / 플러그인
- 새로 추가/수정된 `#[tauri::command]` 함수와 그 인자 타입
- 이벤트 `emit`/`listen` 이름·페이로드 변경

**1-3. 보안 설정 파일 수집 (Tauri 필수)**

아래 파일을 함께 요청한다. diff에 없더라도 권한·CSP 판단에 반드시 필요하다.

| 파일 | 점검 목적 |
|------|-----------|
| `src-tauri/capabilities/*.json` | capability 범위, default 권한, core/플러그인 권한, scope, remote 필드 |
| `src-tauri/tauri.conf.json` | CSP, withGlobalTauri, assetProtocol scope, dangerous* 플래그, 업데이터, 윈도우 구성 |
| `src-tauri/Cargo.toml` | Tauri 버전, 사용 플러그인, 의존성 (cargo audit 대상) |

설정 파일을 받을 수 없으면 해당 영역(권한·CSP)은 "확인 필요"로 명시하고, 받을 수 있는 범위에서만 단정한다.

**1-4. 추가 맥락 (필요시에만)**: Tauri 버전, 타깃 플랫폼(데스크톱/모바일), 타입 생성 도구(tauri-specta 등) 사용 여부.

계획문서·diff·설정 파일이 확보되면 2단계로 이동한다.

---

### 2단계: 분석 및 리뷰 수행

#### 2-1. 계획 대비 정합성 검증

| 검증 항목 | 의미 |
|-----------|------|
| 요구사항 충족 | 계획의 각 요구사항이 코드에 반영되었는가 |
| 누락 구현 | 계획에 있으나 미반영된 항목 |
| 범위 초과 | 계획에 없는 변경(특히 새 커맨드·권한 확대)이 포함되었는가 |

#### 2-2. 보안 핵심 축 우선 점검

체크리스트에 들어가기 전에 두 축을 먼저 통과시킨다.

- **축 ①: 커맨드 입력 검증** — 새/변경 커맨드 인자를 전부 신뢰할 수 없는 입력으로 보고, path traversal·인젝션·범위/길이 체크 누락을 확인한다. (`references/ipc-command-security.md`)
- **축 ②: 권한·스코프 최소화** — capabilities가 최소 권한 원칙을 지키는지, scope가 과도하게 열려 있지 않은지, remote 필드가 외부 origin에 커맨드를 노출하지 않는지 확인한다. (`references/capabilities-scopes.md`)

#### 2-3. 코드리뷰 체크리스트

심각도 순으로 점검한다. 상세 패턴은 `references/`, 점검 항목은 `checklists/` 참조.

**P0 — 치명적 (보안·크래시)**: 머지 전 반드시 해결.

- 커맨드 입력 미검증: path traversal, command/SQL 인젝션, 범위·길이 체크 누락 → 임의 파일/명령 실행
- 위험 프리미티브 노출: 임의 경로 읽기/쓰기, 셸 실행을 가공 없이 커맨드로 노출
- 권한 과다: default capability가 과도하게 넓음, `core:*`/플러그인 권한을 쓰지 않는 것까지 부여
- 위험한 scope: `fs`의 `$HOME/**` 같은 광범위 와일드카드, `shell`의 임의 명령 실행, `http`의 와일드카드 URL
- remote 필드로 외부 origin에 커맨드 노출 (정당 사유 없으면 거부)
- CSP 무력화: `unsafe-inline`/`unsafe-eval`, `dangerousDisableAssetCspModification` 등 dangerous 플래그 활성화
- 프론트엔드 XSS → 권한 상승: `innerHTML`/`dangerouslySetInnerHTML`/동적 HTML 삽입에 살균(sanitize) 누락
- 비밀 노출: 비밀키·토큰이 프론트엔드 번들에 포함, 업데이터 서명용 private key가 저장소/CI 로그에 노출
- 업데이터 결함: HTTP(비HTTPS) 엔드포인트, minisign 서명 검증 비활성/잘못된 pubkey
- 패닉 크래시: 사용자 입력 경로의 `unwrap()`/`expect()`가 코어 프로세스를 죽여 앱 전체 다운
- 동시성: 콜백·핸들러가 같은 락을 중첩 획득하는 데드락, 공유 State 데이터 레이스

**P1 — 잠재적 결함**: 특정 조건에서 문제가 되는 항목.

- async 커맨드 내 블로킹 I/O·CPU 작업을 `spawn_blocking` 없이 호출 → 런타임/이벤트 루프 블로킹
- 에러 누출: `Result` 에러 메시지에 내부 경로·스택·민감 정보 포함
- 공격 표면 확대: 불필요한 `withGlobalTauri`, 과도한 `assetProtocol` scope, 민감 IPC에 isolation pattern 미적용
- 윈도우별 capability 미분리: 권한 낮은 윈도우가 높은 권한 커맨드에 접근
- 외부 콘텐츠: 외부 링크가 시스템 브라우저가 아닌 앱 웹뷰로 열림, 원격 콘텐츠를 웹뷰에 직접 로드
- TS↔Rust 타입 계약 불일치: serde 규약 어긋남, tauri-specta/specta 산출물 미동기화, 이벤트 이름·페이로드 타입 불일치
- 회귀 위험: 기존에 정상 동작하던 기능이 이 변경으로 깨질 수 있는데 회귀 방지 테스트가 없음 — 커맨드 시그니처·serde 계약·이벤트 이름/페이로드 변경(컴파일 에러 없이 기존 프론트 호출부가 조용히 깨짐), capability·scope 축소(기존 기능 권한 부족), 공유 State·락 구조 변경(기존 흐름 동작 변화)이 대표적
- 크로스플랫폼: `#[cfg(...)]` 플랫폼 분기 누락, 모바일 권한 선언 누락, 플랫폼별 경로/웹뷰 차이 미반영
- 의존성 취약점: `cargo audit`(RustSec) 미점검, 취약 크레이트 사용
- `unsafe` 블록의 안전 불변식 주석 부재, `Mutex`/`RwLock` 포이즌 가능성

**P2 — 개선 포인트**: 품질·유지보수성·성능 제안.

- 성능: 대용량 데이터를 JSON IPC로 주고받음 → `tauri::ipc::Channel`/raw payload 미사용, 무거운 작업이 메인 스레드를 막아 UI 프리징
- 에러 처리 일관성: `thiserror`/`anyhow` 미사용, `Result` 반환 패턴 불일치
- 코드 품질: 중복, 과도한 함수 길이, 매직 값, 네이밍
- 테스트: 변경 커맨드/로직 테스트 누락, 엣지 케이스 미고려, 기존 동작을 보호하는 회귀 방지 테스트 유무
- 문서화: 복잡한 IPC 흐름·권한 의도 주석 부재

---

### 3단계: 결과 작성

`templates/review-result.md` 형식을 사용한다. 이 형식이 중요한 이유는 팀이 심각도와 보안 축별로 빠르게 우선순위를 판단하고 조치할 수 있기 때문이다.

#### 출력 형식 (요약)

```
📊 리뷰 요약

| 심각도 | 건수 |
|--------|------|
| 🔴 P0 치명적 | N건 |
| 🟠 P1 잠재적 | N건 |
| 🟡 P2 개선 | N건 |

🔐 보안 핵심 축 점검
- ① IPC 커맨드 입력 검증: [통과 / 위험 / 확인 필요] — 한 줄
- ② 권한·스코프 최소화: [통과 / 위험 / 확인 필요] — 한 줄

✅ 계획 대비 정합성: [충족 / 부분 충족 / 미충족] — 한 줄
```

각 이슈는 아래 형식으로 작성한다:

```
🔴 P0: 치명적 이슈

[P0-1] 이슈 제목
- 📁 파일: `파일경로`
- 📍 위치: `라인 번호 / 커맨드명 / capability 키`
- 🔍 현재 코드/설정:
  (문제가 되는 코드·JSON 발췌)
- ❗ 문제: 구체적 설명 (가능하면 공격 시나리오 명시)
- 💡 수정 제안:
  (권장 코드·설정)
- 📖 근거: 왜 위험한지 기술적 설명
```

P1, P2도 동일 형식. P0·P1에는 반드시 수정 제안을 포함한다. 마지막에 **확인 필요 사항** 표와 **총평**(머지 가부: 승인 / 수정 후 재리뷰 / 수정 필수)을 추가한다.

---

## 제약 조건

- **diff + 설정 범위 한정**: 변경된 코드와 함께 받은 보안 설정 파일을 리뷰한다. 받지 못한 설정 영역은 "확인 필요"로 명시하고 단정하지 않는다.
- **신뢰 경계 가정 고정**: 웹뷰에서 오는 모든 입력은 신뢰할 수 없다고 가정한다. "프론트엔드에서 이미 검증하므로 안전"이라는 논리는 받아들이지 않는다 — 악성 JS가 커맨드를 직접 호출할 수 있기 때문이다.
- **추측 금지**: 코드·설정에서 명확히 확인되지 않는 문제는 "확인 필요" 섹션에 기재한다. 근거 없이 이슈를 만들지 않는다.
- **근거 필수**: 모든 지적에 기술적 근거(가능하면 공격 시나리오)를 포함한다.
- **계획문서 참조**: 계획에 없는 요구사항을 임의로 만들지 않는다.
- **한국어 출력**: 모든 결과는 한국어. 코드·설정 키·기술 용어는 원문 유지.

---

## 참조 파일

리뷰 품질을 높이기 위해 아래 보조 파일들을 포함한다. 리뷰 전에 관련 파일을 참조하면 더 정확한 탐지가 가능하다.

### references/ — 취약점·패턴 참조 가이드
- `ipc-command-security.md`: (축①) 커맨드 인자 검증, path traversal, command/SQL 인젝션, 위험 프리미티브 노출, 에러 누출, async 블로킹
- `capabilities-scopes.md`: (축②) capabilities 최소 권한, default capability, core/플러그인 권한, fs/shell/http scope, remote 필드, 윈도우별 분리
- `webview-csp-security.md`: CSP 엄격성, unsafe-inline/eval, withGlobalTauri, dangerous 플래그, isolation pattern, 프론트엔드 XSS→권한 상승, 외부 콘텐츠/링크
- `rust-code-quality.md`: unwrap/expect 패닉, unsafe 불변식, thiserror/anyhow, cargo audit, Mutex/RwLock 데드락·포이즌, Send/Sync
- `type-contracts-and-platform.md`: TS↔Rust serde 타입 계약, tauri-specta 동기화, 이벤트 계약, 업데이터·코드 서명, 크로스플랫폼(#[cfg], 모바일 권한), IPC 성능(Channel/raw payload)

### templates/ — 출력 템플릿
- `review-result.md`: 리뷰 결과 전체 출력 형식 템플릿
- `context-gathering.md`: 맥락 수집(계획+diff+설정파일) 절차와 정리 양식

### checklists/ — 심각도별 체크리스트
- `p0-critical.md`: P0 치명적 점검 (커맨드 입력, 권한·scope, CSP, XSS→권한상승, 비밀 노출, 업데이터, 패닉, 동시성)
- `p1-potential.md`: P1 잠재적 점검 (async 블로킹, 에러 누출, 공격 표면, 윈도우 분리, 타입 계약, 크로스플랫폼, 의존성)
- `p2-improvement.md`: P2 개선 점검 (IPC 성능, 에러 일관성, 코드 품질, 테스트, 문서화)

---

## 예시

**예시 1: P0 — 커맨드 path traversal (축①)**

```
[P0-1] 파일 읽기 커맨드가 경로 검증 없이 임의 경로를 읽음
- 📁 파일: `src-tauri/src/commands/file.rs`
- 📍 위치: `read_user_file()` 커맨드, 라인 22-26
- 🔍 현재 코드/설정:
  #[tauri::command]
  fn read_user_file(name: String) -> Result<String, String> {
      let path = format!("./data/{}", name);
      std::fs::read_to_string(path).map_err(|e| e.to_string())
  }
- ❗ 문제: 웹뷰의 모든 JS가 이 커맨드를 호출할 수 있다. name에 `../../../../etc/passwd`
  를 넣으면 데이터 디렉터리 밖의 임의 파일을 읽을 수 있다(path traversal).
  XSS가 발생하면 그대로 임의 파일 유출로 이어진다.
- 💡 수정 제안:
  #[tauri::command]
  fn read_user_file(name: String) -> Result<String, String> {
      // 파일명만 허용, 경로 구분자/상위 참조 차단
      if name.contains('/') || name.contains('\\') || name.contains("..") {
          return Err("invalid file name".into());
      }
      let base = std::fs::canonicalize("./data").map_err(|_| "base missing")?;
      let path = base.join(&name);
      let resolved = std::fs::canonicalize(&path).map_err(|_| "not found")?;
      if !resolved.starts_with(&base) { return Err("path escapes base".into()); }
      std::fs::read_to_string(resolved).map_err(|e| e.to_string())
  }
- 📖 근거: 커맨드 인자는 신뢰 경계 그 자체다. canonicalize 후 base 디렉터리
  prefix를 강제하지 않으면 상대 경로·심볼릭 링크로 샌드박스를 벗어날 수 있다.
```

**예시 2: P0 — capability scope 과다 (축②)**

```
[P0-2] fs scope가 $HOME 전체를 와일드카드로 허용함
- 📁 파일: `src-tauri/capabilities/default.json`
- 📍 위치: `fs:scope` 항목
- 🔍 현재 코드/설정:
  { "identifier": "fs:scope", "allow": [{ "path": "$HOME/**" }] }
- ❗ 문제: 홈 디렉터리 전체에 대한 읽기/쓰기를 웹뷰에 위임한 것과 같다.
  fs 플러그인을 호출하는 어떤 경로(XSS 포함)든 사용자의 모든 문서·키·설정에
  접근할 수 있어 권한 통제가 사실상 무력화된다.
- 💡 수정 제안:
  { "identifier": "fs:scope", "allow": [{ "path": "$APPDATA/myapp/**" }] }
  필요한 하위 경로만 추가로 허용하고, deny로 민감 경로를 명시 차단한다.
- 📖 근거: 최소 권한 원칙. fs scope는 앱 데이터 디렉터리 등으로 한정해야 한다.
  광범위 와일드카드는 capability 시스템의 보호 효과를 제거한다.
```

**예시 3: 확인 필요 사항**

```
| # | 질문 | 관련 파일 | 이유 |
|---|------|-----------|------|
| 1 | open_external 커맨드가 받는 URL이 외부 입력인가, 고정 화이트리스트인가? | src-tauri/src/commands/link.rs:14 | 외부 입력이면 임의 스킴(file:, javascript:) 실행 위험. 시스템 브라우저로만 열리는지 확인 필요 |
| 2 | capabilities/default.json을 제공받지 못해 권한 범위를 확정할 수 없음 | (미수신) | 권한·스코프 최소화(축②) 판단을 위해 capability 파일 필요 |
```
