# components

재사용 가능한 presentational 컴포넌트.

- 비즈니스 로직은 feature store / 훅에 두고, 본 디렉터리는 **표현 계층**만 담는다.
- feature 간 공유되는 UI 만 여기 두고, 단일 feature 전용 UI 는 `features/<feature>/components/` 에 둔다.
- Tailwind 우선, 아이콘은 `lucide-react`.
