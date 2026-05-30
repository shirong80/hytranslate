# IPC / 커맨드 경계 보안 참조 가이드 (축①)

`#[tauri::command]`로 노출된 함수는 **신뢰 경계 그 자체**다. 웹뷰에서 실행되는 모든 JS(주입된 악성 코드 포함)가 호출할 수 있다고 가정한다. 커맨드 인자는 전부 신뢰할 수 없는 입력으로 본다. Tauri 2 리뷰에서 사고가 가장 잦은 영역이므로 최우선으로 점검한다.

---

## 1. 경로 순회 (Path Traversal)

### 취약 패턴
```rust
#[tauri::command]
fn read_file(name: String) -> Result<String, String> {
    // ❌ 인자를 그대로 경로에 결합 — ../ 로 샌드박스 탈출 가능
    std::fs::read_to_string(format!("./data/{name}")).map_err(|e| e.to_string())
}
```
`name = "../../../etc/passwd"` 또는 절대경로(`/etc/passwd`, `C:\Windows\...`)를 넣으면 의도한 디렉터리 밖을 읽는다. 심볼릭 링크로도 탈출 가능하다.

### 안전 패턴
```rust
#[tauri::command]
fn read_file(name: String) -> Result<String, String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("invalid file name".into());
    }
    let base = std::fs::canonicalize("./data").map_err(|_| "base missing")?;
    let resolved = std::fs::canonicalize(base.join(&name)).map_err(|_| "not found")?;
    // ✅ canonicalize 후 base prefix 강제 — 심볼릭 링크 포함 탈출 차단
    if !resolved.starts_with(&base) { return Err("path escapes base".into()); }
    std::fs::read_to_string(resolved).map_err(|e| e.to_string())
}
```
핵심: ① 파일명에 경로 구분자/상위 참조 차단, ② `canonicalize()`로 실제 경로 해석 후 base 디렉터리 prefix 강제. 가능하면 fs 플러그인 scope에 위임한다.

---

## 2. 명령어 인젝션 (Command Injection)

### 취약 패턴
```rust
#[tauri::command]
fn convert(input: String) -> Result<String, String> {
    // ❌ 사용자 입력으로 셸 명령 문자열 조립 — `; rm -rf ~` 주입 가능
    let out = std::process::Command::new("sh")
        .arg("-c").arg(format!("ffmpeg -i {input} out.mp4"))
        .output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).into())
}
```

### 안전 패턴
```rust
#[tauri::command]
fn convert(input: String) -> Result<String, String> {
    // ✅ 셸 경유 없이 인자를 분리 전달 — 메타문자가 해석되지 않음
    let out = std::process::Command::new("ffmpeg")
        .args(["-i", &input, "out.mp4"])
        .output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).into())
}
```
`sh -c`로 문자열을 조립하지 않는다. shell 플러그인을 쓰면 `execute`/`open` scope와 sidecar 인자에 사용자 입력이 그대로 들어가지 않는지 확인한다(아래 5).

---

## 3. SQL 인젝션

### 취약 패턴
```rust
// ❌ 문자열 결합으로 쿼리 생성
let q = format!("SELECT * FROM users WHERE name = '{name}'");
sqlx::query(&q).fetch_all(&pool).await?;
```

### 안전 패턴
```rust
// ✅ 파라미터 바인딩
sqlx::query("SELECT * FROM users WHERE name = ?")
    .bind(&name).fetch_all(&pool).await?;
```
`tauri-plugin-sql`을 쓸 때도 프론트엔드에서 넘어온 값을 바인딩 파라미터로 처리하는지 확인한다.

---

## 4. 범위·길이·타입 검증 누락

### 취약 패턴
```rust
#[tauri::command]
fn set_volume(level: i64) { /* ❌ 음수/거대값 검증 없음 */ }

#[tauri::command]
fn alloc_buffer(size: usize) -> Vec<u8> { vec![0; size] } // ❌ size 무제한 → OOM/DoS
```

### 안전 패턴
```rust
#[tauri::command]
fn set_volume(level: i64) -> Result<(), String> {
    if !(0..=100).contains(&level) { return Err("out of range".into()); }
    Ok(())
}

#[tauri::command]
fn alloc_buffer(size: usize) -> Result<Vec<u8>, String> {
    const MAX: usize = 16 * 1024 * 1024;
    if size > MAX { return Err("size too large".into()); }
    Ok(vec![0; size])
}
```
경계값, 길이 상한, 허용 enum 값을 명시 검증한다. 검증을 프론트엔드에만 의존하지 않는다 — 악성 JS가 우회한다.

---

## 5. 위험 프리미티브를 그대로 노출

임의 경로 읽기/쓰기, 임의 셸 실행, eval류 동작을 커맨드로 가공 없이 노출하면 안 된다.

```rust
// ❌ 무엇이든 실행하는 만능 커맨드 — 권한 통제 불가
#[tauri::command]
fn run(cmd: String) -> String { /* shell exec */ }

// ❌ 임의 경로 쓰기
#[tauri::command]
fn write_any(path: String, data: Vec<u8>) -> Result<(), String> { ... }
```
대신 목적이 좁은 커맨드(예: `save_note(id, content)`)로 노출하고, 경로/대상은 앱이 결정하도록 한다. 위험 동작은 플러그인 scope로 통제한다.

---

## 6. 에러 메시지로 내부 정보 누출

### 취약 패턴
```rust
// ❌ 내부 절대경로/스택/DB 구조가 웹뷰로 전달됨
.map_err(|e| format!("failed: {e:?}"))?
```
`e:?`로 OS 에러를 그대로 반환하면 내부 경로, 시스템 구조, 때로는 민감 정보가 프론트엔드(=잠재적 공격자)로 흘러간다.

### 안전 패턴
```rust
// ✅ 사용자 대상 메시지는 일반화, 상세는 서버사이드 로그로만
.map_err(|e| { tracing::error!(?e, "read failed"); "could not read file".to_string() })?
```
`thiserror`로 에러 타입을 정의해 외부 노출용 메시지와 내부 디테일을 분리한다(→ `rust-code-quality.md`).

---

## 7. async 커맨드에서 블로킹 작업 (P1)

### 취약 패턴
```rust
#[tauri::command]
async fn hash_file(path: String) -> Result<String, String> {
    let data = std::fs::read(&path).map_err(|e| e.to_string())?; // ❌ async 안의 블로킹 I/O
    Ok(heavy_cpu_hash(&data)) // ❌ CPU 바운드 작업이 async 런타임 스레드 점유
}
```
async 커맨드 안에서 블로킹 I/O나 CPU 작업을 그냥 호출하면 Tokio 런타임 워커를 막아 다른 IPC가 멈춘다.

### 안전 패턴
```rust
#[tauri::command]
async fn hash_file(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        Ok(heavy_cpu_hash(&data))
    }).await.map_err(|e| e.to_string())?
}
```
블로킹/CPU 작업은 `spawn_blocking`(또는 `tokio::task::spawn_blocking`)으로 분리한다. 동기 커맨드는 기본적으로 별도 스레드에서 실행되지만, 메인 스레드를 요구하는 작업(일부 윈도우 조작)은 예외다.

---

## 리뷰 체크 요약
- [ ] 모든 커맨드 인자를 신뢰 불가 입력으로 보고 검증하는가
- [ ] 경로 인자에 canonicalize + base prefix 강제가 있는가
- [ ] 셸/프로세스 실행에 문자열 조립 대신 인자 분리 전달을 쓰는가
- [ ] SQL은 파라미터 바인딩인가
- [ ] 범위·길이·enum 검증이 Rust 쪽에 있는가 (프론트 의존 금지)
- [ ] 만능 read/write/exec 프리미티브를 노출하지 않는가
- [ ] 에러 메시지에 내부 경로·스택·민감 정보가 없는가
- [ ] async 커맨드의 블로킹/CPU 작업이 spawn_blocking 처리됐는가
