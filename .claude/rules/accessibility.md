# Accessibility

PRD §7.1: "모든 핵심 동작은 키보드로 가능해야 한다."

## Keyboard

- Every interactive element reachable via Tab in logical order
- Focus visible — never `outline: none` without a replacement ring
- **Floating popup**: focus enters input on open; `Esc` closes; `Cmd+Enter` translates; `Cmd+C` copies result when present
- **Main window**: arrow keys navigate the history list; `Cmd+Enter` retranslates
- All custom shortcuts documented in Settings → 단축키 section

## Screen reader (VoiceOver on macOS)

- Semantic HTML: `<button>`, `<textarea>`, `<nav>`, `<main>` — never `<div onClick>`
- `aria-live="polite"` on the translation output region so streaming chunks are announced naturally
- Status indicators have `role="status"` + Korean text label
- Icon-only buttons require `aria-label` in Korean

## Color & contrast

- Text contrast ≥ WCAG AA (4.5:1 normal, 3:1 large)
- Never convey state by color alone — pair with icon or text
- Dark mode tested independently for contrast

## Motion

- Streaming text animates by appearance only — no flying / sliding effects
- Respect `prefers-reduced-motion` for any animation longer than 200ms
