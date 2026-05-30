# 타입 계약 · 업데이터 · 크로스플랫폼 · 성능 참조 가이드

IPC 경계의 타입 일관성, 업데이트/서명, 플랫폼 분기, IPC 성능을 점검한다. 보안 축(①②) 통과 후 본다.

---

## 1. TS ↔ Rust 타입 계약 (serde)

serde 직렬화 규약이 프론트엔드 타입과 어긋나면 런타임에 **조용히** 깨진다.

### 취약 패턴
```rust
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]   // userName 으로 직렬화
struct User { user_name: String }
```
```ts
// ❌ 프론트는 snake_case로 접근 → undefined (컴파일 에러 없이 런타임에 깨짐)
const name = user.user_name;
```

### 안전 패턴
- `#[serde(rename_all = ...)]` 규약과 프론트엔드 타입의 필드명이 일치하는지 본다.
- `Option<T>` ↔ `T | null`/optional, enum의 `#[serde(tag = ...)]` 표현이 프론트 타입과 맞는지 본다.
- **tauri-specta/specta**로 타입을 자동 생성한다면, 산출물(`bindings.ts` 등)이 **최신 상태로 동기화되어 커밋**됐는지 확인한다. 커맨드 시그니처가 바뀌었는데 바인딩이 안 바뀌었으면 P1.
- 수동 타입이면 커맨드 시그니처 변경 시 프론트 타입도 함께 바뀌었는지 본다.

---

## 2. 이벤트 emit / listen 계약

### 취약 패턴
```rust
app.emit("download-progress", payload)?;   // 이름·페이로드 타입이 코드에 흩어짐
```
```ts
listen("download_progress", (e) => {...}); // ❌ 이름 불일치(- vs _) → 영원히 안 옴
```
이벤트 이름 문자열 오타·페이로드 타입 불일치는 자주 새는 부분이다.

### 안전 패턴
- 이벤트 이름을 상수/enum으로 한 곳에서 관리하고 양쪽이 동일 상수를 참조한다.
- 페이로드 타입을 공유(specta 생성 또는 공용 정의)한다.
- emit 측 타입과 listen 측 제네릭 타입이 일치하는지 확인한다.

---

## 3. 회귀 방지 — 기존 동작이 조용히 깨지는 변경

IPC 경계(커맨드·이벤트)와 권한(capability·scope)은 **컴파일러가 호환성을 잡아주지 못한다.** TS↔Rust는 런타임에 JSON으로 연결되고, 권한은 설정 파일로 분리돼 있기 때문이다. 따라서 "기존에 잘 되던 기능"을 깨는 변경이 빌드 성공인 채로 머지될 수 있다 — 전형적 회귀다.

### 조용히 깨지는 대표 변경
```rust
// 변경 전: #[tauri::command] fn save(note_id: u32, body: String)
// 변경 후: 인자명 note_id → id 로 변경
#[tauri::command]
fn save(id: u32, body: String) -> Result<(), String> { ... } // ❌ 기존 invoke('save', { noteId, body }) 호출부는 컴파일 통과, 런타임에 인자 매칭 실패
```
- 커맨드 인자명/타입/추가·삭제 → 변경하지 않은 기존 프론트 호출부가 런타임에 깨짐
- serde `rename`/필드 추가·삭제 → 기존 페이로드 역직렬화가 조용히 실패
- 이벤트 이름·페이로드 변경 → 기존 listen이 영원히 안 옴
- capability/scope 축소·제거 → 그 권한에 의존하던 기존 기능이 권한 부족으로 실패
- 공유 State·락 구조 변경 → 기존 흐름의 동작/타이밍이 바뀜

### 안전 패턴
- 변경이 닿는 공유 표면(커맨드·이벤트·계약·권한·State)을 식별하고, **변경하지 않은 기존 의존부가 함께 갱신됐는지** 확인한다.
- 기존 동작을 고정하는 **회귀 방지 테스트**(직렬화 호환성·커맨드 계약·핵심 흐름 통합 테스트)가 있는지 보고, 없으면 추가를 권고한다.
- 동작 변경이 의도된 것이면 계획문서에 근거가 있는지로 "의도된 변경 vs 의도치 않은 회귀"를 구분한다.
- 가능하면 specta 등으로 타입을 생성해 계약 불일치를 빌드 타임으로 끌어올린다.

---

## 4. 업데이터 · 코드 서명

| 점검 | 안전 기준 |
|------|-----------|
| 엔드포인트 | `https://`만 사용 (평문 HTTP 금지) |
| 서명 검증 | minisign 서명 활성화 + `pubkey` 정확히 설정 |
| **private key** | 저장소·CI 로그·번들에 **절대 노출 금지** (가장 중요) |

```json
// tauri.conf.json — plugins.updater
{ "endpoints": ["https://releases.myapp.com/{{target}}/{{current_version}}"],
  "pubkey": "dW50cnVzdGVk..." }
```
점검: ① `endpoints`가 모두 HTTPS인가, ② `pubkey`가 비어 있지 않고 올바른가, ③ 서명 검증이 비활성화돼 있지 않은가, ④ `TAURI_SIGNING_PRIVATE_KEY` 등 서명 키가 코드/설정/로그/diff에 노출되지 않았는가(시크릿은 CI 시크릿으로만). 키 노출은 P0.

---

## 5. 크로스플랫폼 (데스크톱 + 모바일)

Tauri 2는 iOS/Android까지 대상이다.

### 취약 패턴
```rust
#[cfg(target_os = "windows")]
fn open_path() { /* ❌ macOS/Linux/모바일 분기 누락 → 그 플랫폼에서 미정의/빌드 실패 */ }
```
- 플랫폼 의존 코드의 `#[cfg(...)]` 분기 누락 (한 플랫폼만 처리)
- 경로·파일시스템 차이 미반영 (모바일 샌드박스, 경로 구분자)
- 모바일 권한 선언 누락 (Android `AndroidManifest.xml`, iOS `Info.plist`의 카메라·위치 등)
- 웹뷰 엔진 차이: Windows=WebView2, macOS/iOS=WKWebView, Linux=WebKitGTK → 특정 CSS/JS 기능이 전 플랫폼에서 동작하는지

### 안전 패턴
- 플랫폼 분기는 `#[cfg(...)]` + `#[cfg(not(...))]` 또는 모든 타깃을 커버하는 매칭으로 빠짐없이 처리한다.
- 모바일 타깃이면 필요한 네이티브 권한이 매니페스트에 선언됐는지 확인한다.
- 플랫폼 특화 기능에 폴리필/폴백이 있는지 본다.

---

## 6. IPC 성능

### 취약 패턴
```rust
#[tauri::command]
fn get_big_blob() -> Vec<u8> { read_huge_file() } // ❌ 대용량을 JSON 직렬화로 전송
```
큰 데이터를 IPC로 주고받으면 JSON 직렬화/역직렬화 비용이 크고, 무거운 작업이 메인 스레드/코어를 막아 UI가 프리징된다.

### 안전 패턴
```rust
// ✅ 스트리밍/대용량은 Channel 사용 — 점진 전송, 직렬화 부담 분산
#[tauri::command]
async fn stream_data(on_chunk: tauri::ipc::Channel<Vec<u8>>) -> Result<(), String> {
    for chunk in produce_chunks() { on_chunk.send(chunk).map_err(|e| e.to_string())?; }
    Ok(())
}
```
점검: ① 대용량·스트리밍 데이터에 `tauri::ipc::Channel`이나 raw `Response` payload를 쓰는가, ② 무거운 작업이 `spawn_blocking`/별도 스레드로 분리돼 UI를 막지 않는가(→ `rust-code-quality.md` 4, `ipc-command-security.md` 7), ③ 빈번한 소량 IPC를 배치로 묶을 여지가 없는가.

---

## 리뷰 체크 요약
- [ ] serde rename 규약과 프론트엔드 필드명이 일치하는가
- [ ] specta 등 자동 생성 바인딩이 최신으로 동기화·커밋됐는가
- [ ] 이벤트 이름·페이로드 타입이 양쪽에서 일치하는가
- [ ] 커맨드·계약·권한·State 변경 시 기존 의존부가 깨지지 않거나 함께 갱신됐는가 (회귀)
- [ ] 기존 동작을 고정하는 회귀 방지 테스트가 있는가, 없으면 추가를 권고했는가
- [ ] 업데이터 엔드포인트가 HTTPS이고 서명 검증이 켜져 있는가
- [ ] 서명용 private key가 저장소·CI 로그·diff에 노출되지 않았는가 (P0)
- [ ] 플랫폼 분기 #[cfg]가 누락 없이 처리됐는가
- [ ] 모바일 권한 선언이 매니페스트에 있는가
- [ ] 대용량 데이터에 Channel/raw payload를 쓰는가
- [ ] 무거운 작업이 메인 스레드를 막지 않는가
