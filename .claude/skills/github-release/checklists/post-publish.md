# 게시 후 체크리스트 (Post-publish)

`gh release create` 실행 후 검증한다.

## 릴리스 메타데이터 검증
- [ ] `gh release view v<버전> --json tagName,name,isDraft,isPrerelease,targetCommitish,url` 실행
- [ ] `tagName`이 의도한 태그(`v<버전>`)와 일치
- [ ] `name`이 헤더(제목)와 일치
- [ ] `isDraft` / `isPrerelease`가 의도한 값
- [ ] `targetCommitish`가 의도한 브랜치/커밋
- [ ] `url` 접속 시 본문이 올바르게 렌더링됨 (헤더 중복 노출 없음)

## 태그 무결성
- [ ] `git ls-remote --tags origin v<버전>` 실행
- [ ] 태그 SHA가 태그될 커밋의 SHA와 일치

## 정리·보관
- [ ] 임시 본문 파일 삭제 (`rm /tmp/<버전>-body.md`)
- [ ] `docs/releases/v<버전>.md`를 리포지토리에 커밋 (보관 권장)

## 사전 배포본인 경우
- [ ] `--prerelease`로 게시되어 "Latest"로 표시되지 않음 확인
- [ ] 정식 릴리스 시 다시 `--latest`로 전환할 계획 수립

> 주의: 일부 `gh` 버전에는 `--json isLatest` 필드가 없다. 오류 시 메시지의 사용 가능 필드 목록에서 고른다.
