import { test } from '@playwright/test';

// PRD §14.3 E2E — 이력 검색 + 즐겨찾기 + 삭제.
// assertion plan:
//  1) seed: 3건 번역 후 history 탭 열기.
//  2) FTS5 검색어 입력 → 매칭된 단일 결과로 좁혀짐.
//  3) 항목 클릭 → 우측 detail 패널에 source/translated 표시.
//  4) 즐겨찾기 토글 → favoriteOnly 필터 켜고 해당 항목만 보임을 확인.
//  5) 항목 삭제 → list 에서 사라지고 total -1.
test.skip('history list supports FTS5 search, favorite filter, and delete', () => undefined);
