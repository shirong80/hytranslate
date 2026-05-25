# Styling

## Tailwind

- Use Tailwind utility classes for layout, spacing, color
- Custom classes via `@layer components` only when the same combination repeats 3+ times
- Theme tokens (colors, fonts) live in `tailwind.config.ts` — never hard-code hex values in components
- Dark mode via `darkMode: 'class'`; toggle by setting `dark` on `<html>` based on `Settings.theme`

## Design system

- macOS-native feel: SF Pro font stack, gentle radii (`rounded-md` cards, `rounded` inputs), restrained shadows
- Light / dark / system theme (PRD §7.1, §13). System mode listens to `prefers-color-scheme`
- Use `lucide-react` for icons — clean line aesthetic that matches macOS
- Loading states: subtle spinner or shimmer; no full-screen overlay

## Window-specific styling

- **Main window**: full-height layout, sticky header with model badge + status indicator
- **Floating popup**: 480px default width, max 80% screen height (PRD §6.3), backdrop blur
- **Menubar popover**: compact ~320px width with fixed sections (input / output / clipboard / recent 5)

## Forbidden patterns

- ❌ Inline `style={{ ... }}` for static values — use Tailwind
- ❌ `!important` — almost always a leakage symptom
- ❌ Hardcoded font sizes outside the Tailwind text scale
- ❌ Modal dialogs for errors — use inline error per PRD §7.1
- ❌ Emoji icons in the UI — use `lucide-react`
