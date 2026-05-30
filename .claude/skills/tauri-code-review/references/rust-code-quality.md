# Rust 코드 품질 참조 가이드

코어 프로세스가 죽으면 앱 전체가 다운된다. 패닉·동시성·unsafe·의존성을 점검한다. 특히 웹뷰에서 들어오는 입력 경로에서 더 엄격하게 본다.

---

## 1. unwrap() / expect() 패닉

### 취약 패턴
```rust
#[tauri::command]
fn parse_config(raw: String) -> Config {
    serde_json::from_str(&raw).unwrap() // ❌ 잘못된 입력 → 패닉 → 코어 프로세스 다운
}
```
사용자/웹뷰 입력 경로의 `unwrap()`/`expect()`는 공격자가 의도적으로 패닉을 유발해 앱을 죽이는 DoS가 된다. `Mutex` lock의 `.unwrap()`도 포이즌 시 연쇄 패닉을 일으킨다.

### 안전 패턴
```rust
#[tauri::command]
fn parse_config(raw: String) -> Result<Config, String> {
    serde_json::from_str(&raw).map_err(|e| format!("invalid config: {e}"))
}
```
입력 경계에서는 `?`/`Result`로 에러를 전파한다. `unwrap()`은 "절대 실패하지 않음"이 증명되는 곳(상수, 직전 검증 완료)에만 허용하고, 그 외에는 P0(입력 경로)·P1(기타)로 본다.

---

## 2. unsafe 블록

### 취약 패턴
```rust
// ❌ 안전 불변식 설명 없음 + 검증 안 된 인덱스
unsafe { *ptr.add(i) }
```

### 안전 패턴
```rust
// SAFETY: ptr는 len 길이로 할당된 유효 버퍼이며, 호출부에서 i < len을 보장한다.
unsafe { *ptr.add(i) }
```
`unsafe` 블록에는 정당성과 안전 불변식을 설명하는 `// SAFETY:` 주석이 있어야 한다. 주석이 없거나 불변식이 실제로 보장되지 않으면 지적 대상이다. 가능하면 안전한 추상화로 대체한다.

---

## 3. 에러 처리 일관성 (thiserror / anyhow)

### 권장 패턴
```rust
// ✅ 라이브러리/도메인 에러: thiserror로 타입 정의
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("file not found")]
    NotFound,
    #[error("io error")]              // 외부 노출 메시지는 일반화
    Io(#[from] std::io::Error),       // 내부 디테일은 source로 보존
}

// 커맨드 반환을 위해 serde::Serialize 구현 (Tauri는 Serialize 에러만 반환 가능)
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```
점검: ① 커맨드 에러 타입이 `Serialize`를 구현하는가, ② 외부 노출 메시지에 내부 경로·스택이 새지 않는가(→ `ipc-command-security.md` 6), ③ `Box<dyn Error>`/`anyhow`를 커맨드 경계에서 그대로 노출하지 않는가, ④ 에러 처리 방식이 일관적인가.

---

## 4. 동시성 — 데드락 / 포이즌 / Send·Sync

### 취약 패턴 (중첩 락 데드락)
```rust
fn on_event(state: State<AppState>) {
    let a = state.lock.lock().unwrap();
    notify_handlers();              // ❌ 핸들러가 같은 lock을 다시 획득하면 데드락
}
fn handler(state: State<AppState>) {
    let a = state.lock.lock().unwrap(); // 위 lock 보유 중 재진입 → 멈춤
}
```
콜백·이벤트 핸들러가 이미 보유한 락을 중첩 획득하는 패턴을 특히 본다(과거 동시성 데드락 패턴이 그대로 적용된다). 두 락을 다른 순서로 획득하는 lock-ordering 위반도 데드락 원인이다.

### 안전 패턴
```rust
fn on_event(state: State<AppState>) {
    let snapshot = {
        let a = state.lock.lock().unwrap();
        a.clone()                   // ✅ 락을 짧게 잡고 즉시 해제
    };                              // 스코프 종료로 unlock
    notify_handlers(snapshot);      // 락 미보유 상태에서 콜백 호출
}
```
점검: ① 락 보유 중 콜백/IPC/await를 호출하지 않는가, ② 락 획득 순서가 일관적인가, ③ `std::sync::Mutex`를 async await 구간에 가로질러 잡지 않는가(필요 시 `tokio::sync::Mutex`), ④ `.lock().unwrap()` 포이즌 처리, ⑤ State에 넣는 타입의 `Send`/`Sync` 안전성.

---

## 5. 의존성 취약점 (cargo audit / RustSec)

```bash
cargo audit        # RustSec advisory DB 대조
```
점검: ① 취약 advisory가 있는 크레이트를 쓰는가, ② Tauri/플러그인 버전이 보안 패치가 반영된 버전인가, ③ `Cargo.lock`이 커밋돼 재현 가능한가. diff에 `Cargo.toml` 변경이 있으면 새로 추가된 크레이트의 신뢰성도 본다.

---

## 6. 리소스 / 패널 안전

- 파일/소켓/프로세스 핸들이 누수 없이 정리되는가 (RAII 활용)
- 무한 루프/무제한 채널이 메모리를 잠식하지 않는가
- 패닉이 FFI 경계를 넘어 UB를 일으키지 않는가(`catch_unwind` 필요 지점)

---

## 리뷰 체크 요약
- [ ] 입력 경로에 unwrap()/expect() 패닉이 없는가 (있으면 P0)
- [ ] unsafe 블록에 // SAFETY 주석과 실제 불변식 보장이 있는가
- [ ] 커맨드 에러 타입이 Serialize를 구현하고 내부 정보를 안 흘리는가
- [ ] thiserror/anyhow로 에러 처리가 일관적인가
- [ ] 락 보유 중 콜백/await/IPC 재진입이 없는가 (데드락)
- [ ] async 구간을 가로지르는 std Mutex가 없는가
- [ ] cargo audit 취약점이 없고 Cargo.lock이 커밋됐는가
