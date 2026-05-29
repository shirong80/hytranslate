# 게시 전 체크리스트 (Pre-publish)

릴리스 노트를 작성하고 게시하기 전에 점검한다.

## 버전·태그
- [ ] 버전이 SemVer를 따른다 (`MAJOR.MINOR.PATCH`)
- [ ] 변경 규모와 버전 증가가 맞다 (버그픽스=PATCH / 기능=MINOR / 호환성 깨짐=MAJOR)
- [ ] Git 태그에 `v` 접두사 (`v0.1.0`)
- [ ] 동일 버전의 릴리스/태그가 아직 없다 (`gh release list`, `git tag -l`)

## 매니페스트 동기화
- [ ] 실제 존재하는 버전 매니페스트를 모두 확인했다
- [ ] 매니페스트 `version`이 태그 버전과 일치한다 (`package.json` / `Cargo.toml` / `tauri.conf.json` / `pyproject.toml` / `build.gradle` 등 해당 파일)
- [ ] 불일치가 있었다면 수정 → 커밋 → push를 **태그 전에** 완료했다

## 원격 상태
- [ ] `gh auth status` 통과 (repo 권한)
- [ ] `git remote -v`의 origin이 대상 리포지토리다
- [ ] `git fetch` 후 `git log --oneline origin/main..HEAD`가 비어 있다 (태그될 커밋이 원격에 존재)

## 노트 형식 완비
- [ ] 헤더 1행: 버전 + 이모지 1개 + 테마 문구
- [ ] 태그라인: 한국어 1줄 + 영어 1줄
- [ ] `## 새로운 기능` (한국어, 단일 번호 목록, 임팩트순)
- [ ] `## Changelog` (영어, 1:1 병기)
- [ ] `## 시작하기 / Getting Started`
- [ ] `## 개인정보 / Privacy` (해당 시)
- [ ] `## 시스템 요구사항 / System Requirements`
- [ ] 불릿 본문에 이모지 없음, 명령/단축키는 백틱
