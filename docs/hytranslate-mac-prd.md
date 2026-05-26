# HyTranslate Mac PRD

## 1. 문서 목적

HyTranslate Mac은 macOS 전용 로컬 번역 데스크톱 앱이다. 사용자가 입력하거나 붙여넣은 한국어, 중국어 간체, 중국어 번체 텍스트를 영어로 번역하며, 번역 과정은 사용자의 Mac 안에서만 실행된다.

이 PRD는 개발자와 코딩 에이전트가 전체 v1 제품을 구현할 수 있도록 제품 범위, 사용자 흐름, 화면 요구사항, 데이터 모델, API 계약, 시스템 상태, 예외 처리, 테스트 기준을 정의한다.

## 2. 제품 요약

### 2.1 제품명

HyTranslate Mac

### 2.2 한 줄 설명

HyTranslate Mac은 Tencent Hy-MT2 모델을 Ollama로 로컬 실행하여 한국어와 중국어를 영어로 빠르게 번역하는 macOS 메뉴바 번역 앱이다.

### 2.3 핵심 가치

- 개인정보 보호: 번역 원문과 결과가 사용자의 기기 밖으로 나가지 않는다.
- 빠른 피드백: Ollama streaming 응답을 사용해 번역 결과를 생성되는 즉시 보여준다.
- 오프라인 사용: 모델 다운로드 후 번역 기능은 인터넷 없이 동작한다.
- 작업 흐름 통합: 전역 단축키와 메뉴바 팝업으로 어떤 앱에서든 빠르게 호출한다.
- 반복 사용성: 번역 이력, 검색, 즐겨찾기, 태그로 과거 번역을 재사용한다.

### 2.4 v1 제품 원칙

- v1은 "로컬 번역을 안정적으로 매일 사용할 수 있는 macOS 유틸리티"를 목표로 한다.
- v1에서는 한국어, 중국어 간체, 중국어 번체에서 영어로의 번역만 지원한다.
- v1에서는 계정, 결제, 클라우드 동기화, 서버 저장소를 제공하지 않는다.
- 번역 중 네트워크를 사용하지 않는다. 네트워크는 Ollama 설치 안내, 모델 다운로드, 사용자가 명시적으로 승인한 업데이트에만 사용한다.
- 초기 구현은 Apple Silicon 사용자를 우선한다. Intel Mac은 지원하되 성능 경고를 표시한다.

## 3. 사용자와 문제

### 3.1 주요 사용자

1. 연구자
   - 한국어 논문, 중국어 논문, 기술 자료를 영어로 확인한다.
   - 속도보다 정확성, 전문 용어 보존, 문맥 충실도를 중시한다.

2. 소프트웨어 엔지니어
   - 중국어 코드베이스, 이슈, 메신저 대화를 빠르게 번역한다.
   - 전역 단축키, 클립보드 번역, 낮은 마찰을 중시한다.

3. 법무 및 기밀 문서 사용자
   - NDA가 걸린 계약서, 내부 문서, 민감한 텍스트를 번역한다.
   - 클라우드 번역 금지, 로컬 저장 위치, 이력 삭제 가능 여부를 중시한다.

### 3.2 해결할 문제

- 클라우드 번역기는 원문이 외부 서버로 전송된다.
- 웹 번역기는 앱 전환, 복사, 붙여넣기, 탭 이동이 번거롭다.
- 로컬 LLM 도구는 설치와 모델 준비 과정이 어렵다.
- 일반 LLM은 번역 품질과 출력 형식을 안정적으로 통제하기 어렵다.
- 과거 번역을 검색하거나 재사용하기 어렵다.

## 4. v1 범위

### 4.1 v1에 포함

v1은 초안의 Phase 1부터 Phase 5까지를 포함한다.

- Tauri 2 기반 macOS 데스크톱 앱
- React, TypeScript, Tailwind CSS 기반 프론트엔드
- Rust 기반 백엔드
- Ollama HTTP API 연동
- Hy-MT2 7B 및 1.8B GGUF 모델 사용
- 실시간 streaming 번역
- 한국어, 중국어 간체, 중국어 번체 자동 감지
- 수동 입력 언어 override
- 전역 단축키 호출
- 플로팅 번역 팝업
- 메뉴바 모드
- 클립보드 번역
- SQLite 기반 번역 이력 저장
- 이력 검색, 즐겨찾기, 태그
- 첫 실행 온보딩
- Ollama 설치 감지 및 공식 설치 안내
- 모델 다운로드 진행률 표시
- Ollama 상태 확인 및 재연결
- 한국어 UI
- macOS light/dark/system 테마
- 시작 시 자동 실행 설정
- 기본적인 품질 평가셋 요구사항

### 4.2 v1에서 제외

아래 항목은 v1.1 또는 Future로 미룬다.

- 번역 스타일 선택: Formal, Casual, Literal, Idiomatic
- 사용자 용어집
- 고급 sampling 파라미터 조정 UI
- 통계 대시보드
- Sparkle 자동 업데이트
- UI 다국어화
- OCR 번역
- 선택 텍스트 오버레이 번역
- PDF, DOCX 문서 번역
- 음성 입력
- 영어에서 한국어/중국어로의 역번역
- 로컬 네트워크 동기화
- 계정, 결제, 라이선스 관리
- 클라우드 백업
- SQLite DB 암호화

## 5. 플랫폼 및 기술 스택

### 5.1 지원 환경

- OS: macOS 13 Ventura 이상
- 아키텍처: Apple Silicon 및 Intel Mac
- 배포 형태: 코드 서명 및 notarization된 DMG
- 권장 환경: Apple Silicon, 12GB RAM 이상
- 저사양 환경: 8GB RAM 사용자는 Hy-MT2 1.8B 모델 권장

### 5.2 기술 스택

- Desktop shell: Tauri 2
- Frontend: React, TypeScript, Tailwind CSS, Zustand
- Backend: Rust, Tokio, reqwest, serde, rusqlite
- Model runtime: Ollama
- Model: Tencent Hy-MT2 GGUF
- Local database: SQLite with FTS5
- E2E test: Playwright
- Frontend unit test: Vitest
- Backend unit test: cargo test

### 5.3 외부 의존성 기준

개발 시작 전에 아래 공식 소스에서 버전을 다시 확인한다.

- Ollama API: https://docs.ollama.com/api
- Ollama streaming: https://docs.ollama.com/capabilities/streaming
- Tauri global shortcut plugin: https://v2.tauri.app/reference/javascript/global-shortcut/
- Hy-MT2 7B GGUF: https://huggingface.co/tencent/Hy-MT2-7B-GGUF
- Hy-MT2 1.8B GGUF: https://huggingface.co/tencent/Hy-MT2-1.8B-GGUF

## 6. 핵심 사용자 흐름

### 6.1 첫 실행 온보딩

목표: 비개발자도 문서를 읽지 않고 앱을 사용할 수 있게 한다.

단계:

1. 환영 화면
   - 제품명과 로컬 번역 원칙을 설명한다.
   - "번역 중 원문은 이 Mac 밖으로 전송되지 않습니다."를 명확히 표시한다.

2. 환경 확인
   - macOS 버전, 아키텍처, 메모리, Ollama 설치 여부, Ollama 실행 여부를 확인한다.
   - Intel Mac이면 성능 경고를 표시한다.
   - 12GB 미만 RAM이면 1.8B 모델을 추천한다.

3. Ollama 준비
   - Ollama가 설치되어 있으면 실행 상태를 확인한다.
   - Ollama가 설치되어 있지 않으면 공식 다운로드 링크와 설치 안내를 제공한다.
   - v1에서는 Ollama `.pkg`를 앱에 번들하지 않는다.

4. 모델 선택 및 다운로드
   - 기본 추천은 7B Q4_K_M이다.
   - RAM이 12GB 미만이면 1.8B Q4_K_M을 추천한다.
   - 사용자는 추천 모델을 그대로 진행하거나 다른 모델을 선택할 수 있다.
   - 다운로드 전 예상 용량과 인터넷 연결 필요성을 표시한다.
   - 다운로드는 사용자가 명시적으로 승인해야 시작한다.

5. 권한 안내
   - 전역 단축키와 플로팅 팝업 사용에 필요한 macOS 권한을 안내한다.
   - 필요한 경우 System Settings로 이동할 수 있는 버튼을 제공한다.

6. 이력 저장 안내
   - 기본값은 이력 저장 ON이다.
   - "번역 이력은 이 Mac에만 저장되며 언제든 끄거나 삭제할 수 있습니다."를 표시한다.

완료 조건:

- Ollama가 실행 중이다.
- 선택한 모델이 사용 가능하다.
- 기본 설정이 저장되었다.
- 사용자가 메인 번역 화면으로 이동한다.

### 6.2 메인 창 번역

목표: 긴 텍스트를 안정적으로 입력하고 번역한다.

흐름:

1. 사용자가 메인 창 입력 영역에 한국어 또는 중국어 텍스트를 입력하거나 붙여넣는다.
2. 앱은 500ms 동안 추가 입력이 없으면 자동 번역을 시작한다.
3. 이전 번역 요청이 진행 중이면 즉시 취소한다.
4. 입력 언어를 자동 감지한다.
5. 감지 결과를 UI에 표시한다.
6. 사용자가 원하면 입력 언어를 수동으로 변경한다.
7. Ollama streaming 응답을 받아 출력 영역에 점진적으로 렌더링한다.
8. 번역 완료 시 결과를 이력에 저장한다.

제한:

- 메인 창 입력은 최대 30,000자까지 허용한다.
- 초과 입력은 번역하지 않고 "v1에서는 30,000자까지 지원합니다." 메시지를 표시한다.

### 6.3 전역 단축키 플로팅 팝업

목표: 어떤 앱에서든 즉시 번역할 수 있게 한다.

기본 단축키:

- Cmd+Shift+T

흐름:

1. 사용자가 다른 앱에서 Cmd+Shift+T를 누른다.
2. 앱은 현재 활성 화면 중앙에 플로팅 팝업을 표시한다.
3. 입력 필드에 focus를 둔다.
4. 사용자가 텍스트를 입력하거나 붙여넣는다.
5. 500ms 디바운스 후 번역을 시작한다.
6. 출력 영역에 streaming 결과를 표시한다.
7. Cmd+C를 누르면 번역 결과를 클립보드에 복사한다.
8. Esc를 누르면 팝업을 닫는다.

제한:

- 팝업 입력은 최대 5,000자까지 허용한다.
- 팝업은 폭 480px을 기본값으로 한다.
- 내용이 길어지면 세로로 확장하되 화면 높이의 80%를 넘지 않는다.

### 6.4 클립보드 번역

목표: copy, translate, paste 흐름을 빠르게 만든다.

흐름:

1. 사용자가 팝업 또는 메뉴바에서 "클립보드 번역"을 실행한다.
2. 앱은 텍스트 클립보드를 읽는다.
3. 클립보드가 비어 있거나 텍스트가 아니면 inline 오류를 표시한다.
4. 클립보드 텍스트를 입력 영역에 채우고 번역을 시작한다.
5. 번역 완료 후 사용자는 결과를 클립보드에 복사할 수 있다.

정책:

- 자동으로 원문 클립보드를 덮어쓰지 않는다.
- 설정에서 "번역 완료 후 결과를 클립보드에 자동 복사"를 켤 수 있다.
- 기본값은 OFF이다.

### 6.5 메뉴바 모드

목표: 앱을 Dock에 계속 띄우지 않고도 사용할 수 있게 한다.

요구사항:

- 메뉴바 아이콘을 표시한다.
- 메뉴바 클릭 시 compact popover를 연다.
- popover는 입력 영역, 출력 영역, 클립보드 번역 버튼, 최근 번역 5개를 포함한다.
- 메뉴에는 메인 창 열기, 이력 열기, 설정, 종료가 있어야 한다.
- 설정에서 "Dock 아이콘 숨기기"를 지원한다.

### 6.6 이력 검색

목표: 과거 번역을 다시 찾고 재사용할 수 있게 한다.

흐름:

1. 사용자가 이력 화면을 연다.
2. 최신 번역이 위에 오도록 목록을 표시한다.
3. 사용자가 검색어를 입력하면 source_text와 translated_text를 대상으로 FTS 검색한다.
4. 사용자가 favorite을 토글할 수 있다.
5. 사용자가 tag를 추가, 삭제, 필터링할 수 있다.
6. 사용자가 개별 이력을 삭제할 수 있다.
7. 사용자가 전체 이력을 삭제할 수 있다.
8. 사용자가 CSV 또는 JSON으로 내보낼 수 있다.

## 7. 화면 요구사항

### 7.1 공통 UI 원칙

- UI 기본 언어는 한국어이다.
- macOS native 느낌을 우선한다.
- 시스템 light/dark 모드를 따른다.
- 텍스트 입력과 결과 확인이 중심이며 장식은 최소화한다.
- 모든 핵심 동작은 키보드로 가능해야 한다.
- 오류는 modal alert 대신 해당 영역 안에 inline으로 표시한다.

### 7.2 메인 번역 화면

필수 구성:

- 입력 textarea
- 번역 결과 영역
- 감지된 입력 언어 표시
- 수동 언어 선택
- 번역 상태 표시
- 지연 시간 표시
- 복사 버튼
- 재번역 버튼
- 이력 화면 진입 버튼
- 설정 화면 진입 버튼

상태:

- idle: 입력 대기
- typing: 사용자가 입력 중
- detecting: 언어 감지 중
- translating: 번역 중
- completed: 번역 완료
- cancelled: 이전 요청 취소됨
- error: 오류 발생

### 7.3 플로팅 팝업

필수 구성:

- 입력 textarea
- 출력 영역
- 언어 감지 badge
- 번역 상태 indicator
- 결과 복사 버튼
- 메인 창 열기 버튼

동작:

- 열릴 때 입력 필드에 focus한다.
- Esc로 닫는다.
- Cmd+Enter로 즉시 재번역한다.
- Cmd+C는 출력 결과가 있을 때 결과를 복사한다.
- 출력 결과가 없으면 기본 복사 동작을 방해하지 않는다.

### 7.4 온보딩 화면

필수 구성:

- 단계 indicator
- 환경 확인 결과
- Ollama 설치 상태
- Ollama 실행 상태
- 모델 추천 결과
- 모델 다운로드 진행률
- 이력 저장 안내
- 권한 안내
- 완료 버튼

### 7.5 설정 화면

필수 설정:

- 전역 단축키
- 활성 모델: 7B 또는 1.8B
- Ollama endpoint
- 이력 저장 ON/OFF
- 번역 완료 후 클립보드 자동 복사 ON/OFF
- 시작 시 자동 실행 ON/OFF
- Dock 아이콘 숨기기 ON/OFF
- 테마: 시스템, 라이트, 다크
- 전체 이력 삭제

### 7.6 이력 화면

필수 구성:

- 검색 입력
- 태그 필터
- favorite 필터
- 이력 목록
- 상세 패널
- 복사 버튼
- favorite 버튼
- 태그 편집
- 삭제 버튼
- CSV 내보내기
- JSON 내보내기

## 8. 기능 요구사항

### 8.1 실시간 streaming 번역

요구사항:

- Ollama `/api/generate` endpoint를 사용한다.
- 요청은 `stream: true`로 보낸다.
- 응답 chunk를 받는 즉시 프론트엔드에 전달한다.
- UI는 token 또는 chunk 단위로 결과를 누적 표시한다.
- 입력 변경 후 500ms 동안 추가 입력이 없을 때 번역을 시작한다.
- 사용자가 입력을 변경하면 진행 중인 요청을 취소한다.
- Cmd+Enter는 디바운스를 기다리지 않고 즉시 번역한다.

수용 기준:

- 정상 환경에서 첫 출력 chunk가 도착하기 전까지 UI가 멈추지 않는다.
- 진행 중인 요청 중 입력을 바꾸면 이전 결과가 최종 결과로 저장되지 않는다.
- streaming 중 partial UTF-8 때문에 깨진 문자가 표시되지 않는다.
- 번역 완료 후 duration_ms가 기록된다.

### 8.2 입력 언어 자동 감지

지원 언어:

- Korean
- ChineseSimplified
- ChineseTraditional

감지 방식:

- Hangul Unicode block 비율로 한국어를 판정한다.
- CJK Unified Ideographs 비율로 중국어를 판정한다.
- 중국어 간체/번체는 대표 문자 frequency table로 판정한다.
- 애매한 경우 Auto로 표시하고 모델 prompt에는 `Chinese` 계열 fallback을 사용한다.

정책:

- v1에서는 문장별 혼합 언어 감지를 하지 않는다.
- 혼합 입력은 주 언어를 감지하여 전체 텍스트를 하나의 source language로 번역한다.
- 사용자는 UI에서 감지 결과를 override할 수 있다.

수용 기준:

- 한글만 포함한 입력은 Korean으로 감지한다.
- 간체 중국어 샘플은 ChineseSimplified로 감지한다.
- 번체 중국어 샘플은 ChineseTraditional로 감지한다.
- 사용자가 override하면 이후 번역 요청에는 override 값이 사용된다.

### 8.3 Prompt builder

요구사항:

- source language와 target language를 명시적으로 prompt에 포함한다.
- target language는 v1에서 항상 English이다.
- 모델 출력은 번역문만 반환하도록 지시한다.
- prompt builder는 사용자 입력을 임의로 요약하거나 정규화하지 않는다.

기본 prompt:

```text
Translate the following segment from {source_language} into English.
Output only the translation. Do not add explanations, preambles, quotation marks, or markdown.

{source_text}
```

옵션:

- temperature: 0.3
- top_p: 0.9
- num_predict: 입력 길이에 따라 동적으로 설정하되 기본 512

수용 기준:

- prompt builder unit test가 source language별 결과를 검증한다.
- 사용자의 원문 텍스트가 prompt 안에서 누락되지 않는다.
- target language가 English 외 값으로 설정되지 않는다.

### 8.4 Ollama 상태 관리

요구사항:

- endpoint 기본값은 `http://localhost:11434`이다.
- 앱 시작 시 `/api/tags`로 Ollama 실행 여부와 모델 목록을 확인한다.
- Ollama가 실행 중이 아니면 자동 실행을 시도한다.
- 자동 실행 실패 시 사용자에게 직접 실행 안내를 표시한다.
- 재연결은 exponential backoff를 사용한다.

모델 다운로드:

- 7B 기본 명령: `ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M`
- 1.8B 기본 명령: `ollama pull hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M`
- 다운로드는 사용자가 승인한 경우에만 시작한다.
- 진행률, 받은 용량, 총 용량, 현재 상태를 표시한다.

수용 기준:

- Ollama 미설치 상태에서 앱은 crash하지 않고 설치 안내를 보여준다.
- Ollama 설치 후 재검사 버튼으로 상태가 갱신된다.
- 모델이 없으면 다운로드 화면으로 이동한다.
- 다운로드 중 실패하면 재시도 버튼을 제공한다.

### 8.5 전역 단축키

요구사항:

- 기본 단축키는 Cmd+Shift+T이다.
- 설정에서 변경할 수 있다.
- 이미 등록할 수 없는 단축키는 저장하지 않는다.
- 단축키 등록 실패 시 inline 오류를 표시한다.

수용 기준:

- 앱이 백그라운드에 있어도 단축키로 팝업이 열린다.
- 사용자가 단축키를 변경하면 즉시 새 단축키가 적용된다.
- 앱 종료 시 단축키 등록이 해제된다.

### 8.6 클립보드

요구사항:

- 텍스트 클립보드 읽기를 지원한다.
- 번역 결과를 클립보드에 복사할 수 있다.
- 자동 복사는 설정값에 따른다.
- 기본값은 OFF이다.

수용 기준:

- 빈 클립보드에서는 번역 요청을 보내지 않는다.
- 이미지 또는 파일 클립보드는 v1에서 지원하지 않는다는 메시지를 표시한다.
- 자동 복사 OFF 상태에서는 원문 클립보드가 유지된다.

### 8.7 이력 저장

요구사항:

- 기본값은 이력 저장 ON이다.
- 사용자는 설정에서 이력 저장을 끌 수 있다.
- 이력 저장 OFF일 때 새 번역은 DB에 저장하지 않는다.
- 전체 삭제 기능을 제공한다.
- v1에서는 DB 암호화를 제공하지 않는다.

수용 기준:

- 번역 완료 후 TranslationRecord가 저장된다.
- 취소된 번역은 저장하지 않는다.
- 오류가 발생한 번역은 저장하지 않는다.
- 이력 저장 OFF 상태에서는 완료된 번역도 저장하지 않는다.

## 9. 데이터 모델

### 9.1 TranslationRecord

| 필드 | 타입 | 필수 | 설명 |
|---|---:|---:|---|
| id | TEXT | Y | UUID |
| source_text | TEXT | Y | 원문 |
| source_language | TEXT | Y | Korean, ChineseSimplified, ChineseTraditional, Auto |
| translated_text | TEXT | Y | 영어 번역 결과 |
| model | TEXT | Y | 사용 모델 |
| duration_ms | INTEGER | Y | 번역 소요 시간 |
| created_at | TEXT | Y | ISO 8601 timestamp |
| is_favorite | INTEGER | Y | 0 또는 1 |
| tags_json | TEXT | Y | JSON string array |

### 9.2 Settings

| 필드 | 타입 | 기본값 | 설명 |
|---|---:|---:|---|
| global_hotkey | TEXT | Cmd+Shift+T | 전역 단축키 |
| active_model | TEXT | Hy-MT2-7B | 활성 모델 |
| auto_copy_after_translation | INTEGER | 0 | 번역 완료 후 자동 복사 |
| save_history | INTEGER | 1 | 이력 저장 |
| start_at_login | INTEGER | 0 | 로그인 시 시작 |
| hide_dock_icon | INTEGER | 0 | Dock 아이콘 숨김 |
| ollama_endpoint | TEXT | http://localhost:11434 | Ollama endpoint |
| theme | TEXT | System | System, Light, Dark |
| onboarding_completed | INTEGER | 0 | 온보딩 완료 여부 |

### 9.3 ModelInstallState

| 필드 | 타입 | 설명 |
|---|---:|---|
| model_id | TEXT | 모델 식별자 |
| display_name | TEXT | UI 표시 이름 |
| ollama_name | TEXT | Ollama 모델 이름 |
| installed | INTEGER | 설치 여부 |
| recommended | INTEGER | 현재 하드웨어 기준 추천 여부 |
| last_checked_at | TEXT | 마지막 확인 시각 |

> v1 노트 (2026-05-26 code review v1 follow-up §10):
> v1 에서는 `ollama_name` / `installed` / `recommended` 를 `/api/tags` 런타임 응답으로 대신하고,
> `last_checked_at` 만 `settings.modelInstallState` 에 영속화한다. throttle: 5분. DB schema
> 자체는 추가하지 않으며 v1.1 에서 본 표가 필요해지면 재논의.

### 9.4 SQLite 요구사항

- DB 위치: `~/Library/Application Support/HyTranslate Mac/hytranslate.sqlite`
- FTS5를 사용해 source_text와 translated_text를 검색한다.
- DB migration 체계를 둔다.
- DB schema version을 저장한다.

## 10. Backend command 계약

Tauri command 이름은 구현 중 변경할 수 있으나, 아래 기능 경계는 유지해야 한다.

### 10.1 translate_stream

입력:

```json
{
  "sourceText": "안녕하세요.",
  "sourceLanguage": "Korean",
  "model": "hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M",
  "requestId": "uuid"
}
```

이벤트:

- `translation:started`
- `translation:chunk`
- `translation:completed`
- `translation:cancelled`
- `translation:error`

### 10.2 cancel_translation

입력:

```json
{
  "requestId": "uuid"
}
```

동작:

- 진행 중인 해당 request를 취소한다.
- 취소된 request의 결과는 DB에 저장하지 않는다.

### 10.3 detect_language

입력:

```json
{
  "text": "입력 텍스트"
}
```

출력:

```json
{
  "language": "Korean",
  "confidence": 0.94
}
```

### 10.4 get_ollama_status

출력:

```json
{
  "installed": true,
  "running": true,
  "endpoint": "http://localhost:11434",
  "models": ["hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M"]
}
```

### 10.5 pull_model

입력:

```json
{
  "model": "hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M"
}
```

이벤트:

- `model-pull:started`
- `model-pull:progress`
- `model-pull:completed`
- `model-pull:error`

### 10.6 history commands

필수 command:

- `list_translation_records`
- `search_translation_records`
- `get_translation_record`
- `delete_translation_record`
- `delete_all_translation_records`
- `toggle_favorite`
- `set_tags`
- `export_history_csv`
- `export_history_json`

## 11. 오류 상태와 메시지

### 11.1 Ollama 미설치

조건:

- `ollama` binary를 찾을 수 없다.
- `localhost:11434`에 연결할 수 없다.

메시지:

```text
Ollama가 설치되어 있지 않습니다. HyTranslate Mac은 로컬 번역을 위해 Ollama가 필요합니다.
```

액션:

- 공식 다운로드 페이지 열기
- 다시 확인

### 11.2 Ollama 미실행

메시지:

```text
Ollama가 실행 중이 아닙니다. 자동 실행을 시도하거나 직접 실행해 주세요.
```

액션:

- 자동 실행 시도
- 다시 연결

### 11.3 모델 없음

메시지:

```text
선택한 번역 모델이 아직 다운로드되지 않았습니다.
```

액션:

- 추천 모델 다운로드
- 다른 모델 선택

### 11.4 입력 길이 초과

메시지:

```text
현재 화면에서는 최대 {limit}자까지 번역할 수 있습니다.
```

### 11.5 번역 실패

메시지:

```text
번역 중 문제가 발생했습니다. Ollama 상태를 확인한 뒤 다시 시도해 주세요.
```

액션:

- 다시 시도
- Ollama 상태 보기

### 11.6 권한 문제

메시지:

```text
전역 단축키를 사용하려면 macOS 권한 설정이 필요합니다.
```

액션:

- System Settings 열기
- 나중에 하기

## 12. 보안 및 개인정보

### 12.1 개인정보 원칙

- 번역 요청은 기본적으로 localhost Ollama endpoint로만 전송한다.
- 원문과 번역 결과를 외부 서버로 전송하지 않는다.
- 원격 telemetry는 v1에서 제공하지 않는다.
- 로그는 로컬에만 저장한다.
- 로그에 원문과 번역 결과를 기본 포함하지 않는다.

### 12.2 로컬 저장

- 이력 저장은 기본 ON이다.
- 첫 실행에서 로컬 저장 사실을 명확히 안내한다.
- 사용자는 이력 저장을 끌 수 있다.
- 사용자는 전체 이력을 삭제할 수 있다.
- v1에서는 SQLite DB 암호화를 제공하지 않는다.

### 12.3 네트워크 사용

허용되는 네트워크 사용:

- 사용자가 승인한 모델 다운로드
- 사용자가 직접 연 Ollama 공식 설치 페이지
- 향후 사용자가 승인한 업데이트 확인

금지:

- 번역 원문 외부 전송
- 번역 결과 외부 전송
- default-on telemetry
- 사용자 동의 없는 업데이트 확인

## 13. 성능 요구사항

### 13.1 UI 성능

- 번역 중 입력 UI가 멈추지 않아야 한다.
- streaming chunk 렌더링은 UI flicker를 만들지 않아야 한다.
- 큰 입력에서도 스크롤과 복사가 가능해야 한다.

### 13.2 번역 성능

하드웨어와 모델에 따라 속도가 달라지므로 절대 시간을 보장하지 않는다. 대신 앱은 아래 상태를 투명하게 보여준다.

- 첫 token 또는 첫 chunk 대기 중
- 번역 진행 중
- 완료
- duration_ms
- 모델명

### 13.3 메모리 정책

- 12GB 이상 RAM: 7B 모델 추천
- 12GB 미만 RAM: 1.8B 모델 추천
- Intel Mac: 성능 경고 표시

## 14. 품질 평가

### 14.1 평가셋

v1 개발 중 `evals/translation-quality.md` 파일을 별도로 생성한다.

구성:

- 한국어 40개
- 중국어 간체 40개
- 중국어 번체 20개

도메인:

- 법률 및 계약
- 학술 및 연구
- 소프트웨어 개발
- 비즈니스 커뮤니케이션
- 일상 표현

### 14.2 평가 기준

5점 척도:

- 5: 의미, 문체, 용어가 모두 자연스럽고 정확하다.
- 4: 업무에 사용할 수 있으며 사소한 어색함만 있다.
- 3: 대체로 이해 가능하지만 수정이 필요하다.
- 2: 중요한 의미 일부가 손상되었다.
- 1: 오역 또는 사용 불가.

v1 통과 기준:

- 전체 평균 4.0 이상
- 치명적 오역 5% 이하
- 법률/학술 샘플의 주요 용어 보존 실패 10% 이하
- 한국어, 간체, 번체 각각 평균 3.8 이상

### 14.3 구현 평가

필수 테스트:

- Prompt builder unit test
- Language detection unit test
- Ollama client mock streaming test
- Translation cancellation test
- SQLite migration test
- History search test
- Clipboard command test
- Settings persistence test
- Onboarding state transition test
- Playwright 기반 주요 사용자 흐름 E2E test

## 15. 개발 로드맵

### 15.1 Phase 1: 핵심 번역 루프

목표:

- Ollama가 이미 설치되어 있고 모델이 준비된 사용자가 메인 창에서 번역할 수 있다.

필수 작업:

- Tauri 2 프로젝트 scaffold
- React/TypeScript UI 구성
- Rust command bridge 구성
- Ollama streaming client 구현
- 단일 메인 창 입력/출력 UI 구현
- 번역 요청 취소 구현
- light/dark/system 테마 적용

완료 기준:

- 한국어 또는 중국어 입력을 영어로 streaming 번역한다.
- 입력 변경 시 이전 요청이 취소된다.
- 번역 완료 시간이 표시된다.

### 15.2 Phase 2: 언어 감지와 설정

목표:

- 사용자가 입력 언어를 직접 고르지 않아도 번역할 수 있다.

필수 작업:

- 언어 감지 함수 구현
- 수동 언어 override UI
- prompt builder 구현
- 설정 화면 scaffold
- 설정 저장 구현

완료 기준:

- 한국어, 간체, 번체 샘플을 감지한다.
- override 값이 prompt에 반영된다.
- 기본 설정이 앱 재시작 후 유지된다.

### 15.3 Phase 3: macOS 시스템 통합

목표:

- 앱을 어디서든 단축키와 메뉴바로 사용할 수 있다.

필수 작업:

- 전역 단축키 등록
- 플로팅 팝업 창 구현
- 메뉴바 아이콘 및 popover 구현
- 클립보드 읽기/쓰기 구현
- 시작 시 자동 실행 설정 구현
- Dock 아이콘 숨김 설정 구현

완료 기준:

- Cmd+Shift+T로 팝업을 열 수 있다.
- 클립보드 텍스트를 번역할 수 있다.
- 메뉴바에서 compact 번역을 실행할 수 있다.

### 15.4 Phase 4: 이력과 검색

목표:

- 번역 결과를 로컬에 저장하고 다시 찾을 수 있다.

필수 작업:

- SQLite schema 및 migration 구현
- TranslationRecord 저장 구현
- FTS5 검색 구현
- 이력 화면 구현
- favorite, tag, 삭제 구현
- CSV/JSON export 구현
- 이력 저장 ON/OFF 구현

완료 기준:

- 완료된 번역이 이력에 저장된다.
- 검색, favorite, tag 필터가 동작한다.
- 전체 삭제가 동작한다.
- 이력 저장 OFF에서는 새 기록이 저장되지 않는다.

### 15.5 Phase 5: 온보딩과 모델 lifecycle

목표:

- 비개발자도 앱을 설치하고 모델을 준비해 사용할 수 있다.

필수 작업:

- 첫 실행 온보딩 구현
- 환경 감지 구현
- Ollama 설치 여부 확인
- Ollama 실행 상태 확인 및 자동 실행 시도
- 공식 설치 안내 제공
- 모델 추천 구현
- 모델 다운로드 진행률 구현
- 오류 상태 및 재시도 UI 구현

완료 기준:

- Ollama가 없는 사용자는 설치 안내를 받을 수 있다.
- 모델이 없는 사용자는 앱 안에서 다운로드를 시작할 수 있다.
- 다운로드 실패 시 재시도할 수 있다.
- 온보딩 완료 후 정상 번역 화면으로 진입한다.

## 16. v1.1 이후 후보

우선순위 후보:

- 번역 스타일: Formal, Casual, Literal, Idiomatic
- 사용자 용어집
- 설정에서 7B/1.8B 전환 고도화
- 30B-A3B 고급 모델 옵션
- 번역 통계
- Sparkle 자동 업데이트
- UI 한국어/중국어/영어 다국어화

## 17. Future 후보

- OCR 영역 캡처 번역
- 선택 텍스트 tooltip 번역
- PDF/DOCX 문서 번역
- 음성 입력
- 영어에서 한국어/중국어로 역번역
- Ollama 호환 임의 모델 선택
- 로컬 네트워크 기반 이력 동기화

## 18. 명시적으로 닫힌 의사결정

| 항목 | 결정 |
|---|---|
| 제품명 | HyTranslate Mac |
| v1 범위 | Phase 1~5 |
| 첫 구현 대상 | 전체 v1 |
| 실시간 번역 정책 | 500ms 디바운스, 이전 요청 취소, Cmd+Enter 즉시 재번역 |
| 팝업 입력 제한 | 5,000자 |
| 메인 창 입력 제한 | 30,000자 |
| 이력 저장 | 기본 ON |
| 클립보드 자동 덮어쓰기 | 기본 OFF |
| Ollama 설치 | v1은 공식 설치 안내와 감지, 번들 `.pkg` 제외 |
| 모델 다운로드 | 앱 내부에서 사용자 승인 후 실행 |
| 인터넷 정책 | 번역 중 네트워크 사용 없음 |
| UI 언어 | 한국어 |
| macOS 지원 | macOS 13 이상 |
| DB 암호화 | v1 제외 |
| 입력 언어 | 한국어, 중국어 간체, 중국어 번체 |
| 출력 언어 | 영어 |
| 계정/결제/라이선스 | v1 제외 |

## 19. Definition of Done

v1은 아래 조건을 모두 만족해야 완료로 본다.

- macOS 13 이상에서 앱이 설치 및 실행된다.
- 첫 실행 온보딩을 통해 Ollama와 모델 준비 상태를 확인할 수 있다.
- 선택한 Hy-MT2 모델로 한국어/중국어 입력을 영어로 번역한다.
- streaming 결과가 UI에 점진적으로 표시된다.
- 입력 변경 시 이전 번역 요청이 취소된다.
- 전역 단축키로 플로팅 팝업을 열 수 있다.
- 메뉴바에서 번역 기능을 사용할 수 있다.
- 클립보드 번역이 동작한다.
- 번역 이력이 SQLite에 저장되고 검색된다.
- 이력 저장을 끄고 전체 삭제할 수 있다.
- 주요 오류 상태가 inline으로 표시된다.
- 번역 원문과 결과가 외부 서버로 전송되지 않는다.
- 필수 unit, integration, E2E 테스트가 통과한다.
- 품질 평가셋 기준을 만족한다.
