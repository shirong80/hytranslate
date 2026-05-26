# features

feature slice 모음. `.claude/rules/architecture.md` 의 "feature 가 store + ipc + types 를 소유한다" 원칙을 따른다.

레이아웃:

```
features/<feature>/
├── components/   # feature 전용 UI
├── store.ts      # Zustand store (selector 패턴)
├── ipc.ts        # invoke/listen 래퍼
└── types.ts      # feature 도메인 타입
```

규칙: 다른 feature 의 store 를 직접 import 하지 않는다. 컴포넌트 계층에서 selector 로 조합한다.
