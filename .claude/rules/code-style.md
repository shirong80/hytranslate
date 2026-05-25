# React / TypeScript Style

## Components

- Function components only; props typed as a named `interface <Component>Props`
- Default-export at most one component per file; named-export sub-components
- Hooks called unconditionally at the top of the component
- Reach for `memo()` only after a perf measurement justifies it

## Hooks

- `useXxx` naming; return a tuple or named object — never a positional array of >3 items
- Feature-local hooks next to the feature; cross-feature hooks in `src/lib/hooks/`
- Cleanup in `useEffect` is mandatory whenever a subscription / listener is created

## State (Zustand)

- One store per feature: `create<State>()(set => ({ ... }))`
- Use selector functions: `const x = useStore(s => s.x)` — never destructure the whole store
- Persist only Settings via `zustand/middleware` `persist()`; never persist in-flight translation state
- Actions colocated inside the store object

## TypeScript

- `strict: true` — no `any`, no `// @ts-ignore` (use `// @ts-expect-error <reason>` if absolutely necessary)
- `interface` for object shapes, `type` for unions / aliases
- Discriminated unions for state machines (`status: 'idle' | 'translating' | ...`)
- Branded types for IDs: `type RequestId = string & { __brand: 'RequestId' }`
- Avoid `enum` — use `as const` object + union type

## Formatting

- Prettier defaults: 2-space indent, single quotes, semicolons, trailing commas (es5)
- Line width 100
- Import order: builtin / external / `@/`-aliased / relative — separated by blank lines

## Naming

- Files: `kebab-case.tsx` for components, `kebab-case.ts` for modules
- Components: `PascalCase`
- Hooks / functions / variables: `camelCase`
- Constants: `SCREAMING_SNAKE_CASE` only for true module-level constants
- Tauri event names: defined once in `src/lib/ipc/events.ts`, never inline string literals
