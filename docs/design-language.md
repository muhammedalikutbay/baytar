# Baytar design language

## Purpose and principles

Baytar is an offline desktop work tool for independent veterinarians and small clinics.
Keep the interface plain, calm and predictable: white by day, near black by night.
Borrow Menura's Inter, hairline edges, generous spacing, soft corners and neutral primary buttons.
Use denser lists suitable for clinical work. Never use marketing heroes, gradients, glows,
italic serif type, decorative animation or oversized display headings.

- Give each screen one primary action. Use ink for it; supporting actions are outline or ghost.
- Put information before decoration. Group related details with space and quiet hairlines.
- Keep table rows compact, labels short and actions in consistent locations.
- Be honest about missing functionality. The initial pages are placeholders; their dialogs explain
  the planned forms and never pretend to save data. Home links to the patient list.
- Data remains on the user's computer. Do not use remote fonts, telemetry or network dependencies at runtime.

## Semantic color tokens

The source is `app/src/app.css`, using Tailwind v4 `@theme static`.
The default Tailwind color palette is removed. Components use semantic utilities or variables,
never raw hex, named palette colors or one-off color literals.
Theme overrides live on `[data-theme='light']` and `[data-theme='dark']`; nested previews work too.

| Token | Light | Dark | Role |
|---|---|---|---|
| `--color-canvas` | `#F7F7F9` | `#0B0B0F` | Page canvas |
| `--color-surface` | `#FFFFFF` | `#141418` | Cards, sidebar, top bar, fields, dialogs |
| `--color-surface-muted` | `#F2F2F5` | `#1C1C22` | Selected rows, hover and disabled controls |
| `--color-border` | `#E9E9EE` | `#26262E` | Decorative hairline only |
| `--color-border-strong` | `#C9C9D2` | `#3A3A44` | Decorative stronger line only |
| `--color-border-control` | `#85858F` | `#6B6B76` | Control boundary, minimum 3:1 |
| `--color-fg` | `#0B0B0F` | `#F2F2F5` | Body text and headings |
| `--color-fg-muted` | `#62626C` | `#A1A1AA` | Secondary text |
| `--color-fg-subtle` | `#6E6E78` | `#858590` | Placeholders and auxiliary text |
| `--color-ink` | `#111114` | `#F2F2F5` | Primary button background; checked icons |
| `--color-ink-fg` | `#FFFFFF` | `#0B0B0F` | Text on ink only |
| `--color-accent` | `#11756F` | `#4FC3B8` | Links, focus ring, tiny brand icon |
| `--color-highlight` | `#A9521C` | `#F0A06A` | Small emphasis and warning labels |
| `--color-success` | `#1F7A4D` | `#5BC48A` | Success text and icons |
| `--color-danger` | `#C42B1C` | `#F27A6E` | Error text and icons |

Soft badge colors are opaque `color-mix(in srgb, var(--color-ROLE) 10%, var(--color-surface))`
for accent, highlight, success and danger. This fixes the background regardless of the parent.
Use these tints for small badges only. Neutral badges use surface-muted with fg-muted text.
The overlay is `rgb(11 11 15 / .42)`, a decorative modal scrim without text.

### Contrast adjustments from the requested starting palette

Three values were adjusted; every other supplied palette value is preserved.

| Token | Requested | Final | Reason |
|---|---|---|---|
| Light fg-subtle | #8E8E98 | #6E6E78 | Small placeholders must pass 4.5:1 on surface-muted (original 2.89:1). |
| Dark fg-subtle | #7A7A85 | #858590 | Small helper text must pass 4.5:1 on surface-muted (original 4.03:1). |
| Light border-control | #8E8E98 | #85858F | Boundary must pass 3:1 on surface-muted (original 2.89:1). |

Do not restore the original subtle values for small text. Thin decorative border and border-strong
tokens are intentionally low contrast; they cannot identify a form control, selected state or essential icon.

### Accent rules

Allowed:

- A small turquoise paw next to the black/white Baytar wordmark.
- An underlined turquoise link, a 2 px selected navigation bar, a focus outline or a small icon.
- A short orange warning label or one word of emphasis.
- Small labeled status badges with their matching soft tint. The text communicates the state.

Forbidden:

- Turquoise or orange page, card, sidebar, banner or dialog backgrounds.
- Turquoise or orange primary buttons, large filled illustrations or large tinted sections.
- Gradients, glows, large colored panels, tinted form fields or colored toast backgrounds.
- Status shown by color alone.

Danger buttons are surface-filled with a danger outline and danger text. Success and danger use
small text/icons/badges; they do not introduce large fills. A checkbox uses a neutral checked mark.
Text selection uses ink with ink-fg, not an unmeasured translucent accent fill.

## Typography

Bundle `@fontsource-variable/inter/wght.css`, including its Latin and Latin Extended subsets for Turkish.
All font URLs resolve locally through Vite. No CDN, remote CSS or runtime font request.
The redistributed SIL Open Font License is included at `app/static/fonts/Inter-OFL.txt`.
Fallback: Inter, ui-sans-serif, system-ui, sans-serif. Enable `ss01` and `cv11`.
Use regular 400, medium 500 and semibold 600; no serif or italic emphasis.

| Utility | Size | Line height | Weight | Tracking |
|---|---:|---:|---:|---:|
| type-page | 28 px | 1.25 | 600 | -0.035 em |
| type-title | 20 px | 1.35 | 600 | -0.025 em |
| type-section | 15 px | 1.5 | 600 | -0.01 em |
| type-body (default) | 14 px | 1.55 | 400 | normal |
| type-caption | 12 px | 1.5 | 500 | normal |

Use tabular numerals for doses, dates, times and aligned numeric columns. Body line length should
usually remain below 80 characters. Never use subtle or muted text on ink; ink-fg belongs there.

## Space, shape and elevation

Use a 4 px base: 4, 8, 12, 16, 20, 24, 32, 40 and 48 px. Micro gaps may use 6 or 10 px.
Page blocks are 32 px apart, card padding 24 px (20 px in narrow contexts), fields 16 px apart.
Lists use about 44 px rows; avoid both cramped controls and marketing-sized empty areas.

The sidebar is 232 px, collapsing to a 76 px icon rail below 1100 px.
Its neutral selected fill includes a 2 px accent bar and `aria-current="page"`.
The content region is at most 1200 px inside 32 px gutters (24 px narrower, 16 px on small windows).
The shell uses an 88 px top bar with the current page and global search; small windows stack these.
Settings and the three theme buttons stay at the bottom of the sidebar.

| Radius token | Size | Use |
|---|---:|---|
| sm | 10 px | Buttons, nav rows, badges |
| md | 14 px | Inputs, inset groups, toast |
| lg | 20 px | Cards, empty-state icon frame |
| xl | 28 px | Dialogs and gallery groups |
| pill | 9999 px | Tiny selection indicators, occasional compact chips |

Small 5 px micro corners are permitted for checkbox squares and keyboard keycaps.

Hairline shadow: `0 0 0 1px var(--color-border)`.
Float: two very soft layers for dialogs/toasts only; see CSS for exact values.
Dark float shadows have .08/.18 opacity and remain quiet against near-black surfaces.
Resting cards use a 1 px border and no shadow. Do not stack hairline shadow with a card border.

Motion is limited to 160 ms color changes. Respect reduced motion; no entrance or continuous animations.

## Component contract

All reusable primitives live in `app/src/lib/components/ui/`, use Svelte 5 runes and accept
native attributes where appropriate. `cn` composes clsx and tailwind-merge.
Keep labels, help text and error messages connected to their controls.

| Component | Contract |
|---|---|
| Button | primary ink/ink-fg, secondary control outline, ghost transparent, danger surface/outline; sm 36 px and md 44 px; native button type defaults to button. |
| IconButton | 44 px target; Turkish accessible label and tooltip required; selected neutral fill, accent icon and aria-pressed supplied by caller. |
| Input | 44 px minimum; surface, border-control; label required, bindable value/ref, linked hint/error and aria-invalid. |
| Textarea | Same field contract; vertical resizing, at least 96 px. |
| Select | Native single select for platform keyboard behavior; labeled, bindable string, options, disabled/error states. |
| Checkbox | bits-ui state/keyboard/form semantics; 24 px box and 44 px label row; neutral check and mixed indicator; no color-only state. |
| Card | Surface, lg radius, hairline border, 20/24 px padding. |
| Badge | Neutral, accent, highlight, success, danger; text label required; only small matching soft fills. |
| EmptyState | Neutral icon frame, h2 title, one sentence and one action snippet. |
| Dialog | bits-ui portal, focus trap, title/description, Escape/outside close, scroll lock and focus restoration; max 512 px, 85dvh scrolling. |
| Toast | Polite live region, surface/float, dismiss control, optional action snippet for undo; persistent until dismissed. No timed loss of undo. |
| Kbd | Neutral keycap; shortcuts remain understandable without it. |
| Separator | Decorative hairline with horizontal/vertical separator semantics. |

Icons use Lucide, normally 18–20 px, 1.75 stroke. Icons alongside visible text are aria-hidden.
Use text-fg-muted for most icons; the wordmark, focus and compact selected theme icon may use accent.
Disabled controls stay legible with fg-subtle on surface-muted; avoid opacity that undermines measured contrast.
Button hover uses an underline on primary/danger and surface-muted on outline/ghost.
Do not disable a record action just because there are no records; explain the next step.

## Theme and browser behavior

`theme.svelte.ts` exposes the rune-backed preference, resolved mode, setter and initialization cleanup.
Preferences: system, light, dark. Persist to `localStorage['baytar.theme']`.
The inline script in `app.html` validates storage and sets `<html data-theme>` before the CSS/app paints.
System mode follows `prefers-color-scheme`, including live OS changes. Cross-window storage changes sync.
Unavailable storage falls back safely to system; the current window's controls still work.

The root layout initializes the theme once and removes listeners on teardown.
All Tauri calls are guarded with `'__TAURI_INTERNALS__' in window`; normal browsers render without IPC.
Search has no backend yet. Ctrl+K (also Cmd+K) focuses its input; Enter opens an explanatory dialog.
Global search never steals focus from an open modal.

## Accessibility and Turkish copy

Normal text must pass WCAG AA 4.5:1; large text (24 px regular or 18.66 px bold) and essential UI
boundaries/icons need 3:1. All defined text colors meet the stricter 4.5:1 on all three neutral surfaces.
The decorative hairlines are exempt because they do not communicate essential control boundaries.
A 2 px accent focus-visible outline with 3 px offset sits against the surrounding neutral surface,
including around ink buttons. Keep it unclipped. A high-contrast OS mode uses system Highlight.
Include keyboard order, skip-to-content, native form semantics, button labels and modal focus containment.

Tab and Shift+Tab move through controls; Enter/Space activate buttons and checkboxes; arrow keys operate
native selects; Esc dismisses dialogs. Theme buttons use a labeled group and aria-pressed.
Small buttons are a deliberate 36 px desktop density option, exceeding the WCAG 2.2 24 px target minimum;
default controls and icon buttons are 44 px. Never make essential icons depend on a hover tooltip.

All product copy lives in `app/src/lib/i18n/tr.ts`; identifiers and engineering docs remain English.
Use Turkish sentence case, correct dotted/dotless I, short nouns and clear verbs: “Hasta ekle”,
“Randevu ekle”, “Vazgeç”, “Geri al”. Avoid ALL CAPS, title case, jargon and long button labels.
Use one sentence in empty-state descriptions. Error copy explains what to fix without blame.
Use Turkish-aware formatting for real dates and numbers when those modules arrive.

## Measured contrast reference

Computed using WCAG sRGB relative luminance: linearize each channel, apply
0.2126 R + 0.7152 G + 0.0722 B, then (lighter + .05) / (darker + .05).
Ratios below are rounded to two decimals after calculating with full precision.
Columns cover normal, hover, selected, disabled, placeholder and neutral badge surfaces.
Ink on surfaces also covers checkbox checks; ink-fg is used only on ink.
Soft pairs use the exact 10% sRGB mix, not an alpha estimate.

### Light contrast

| Foreground | Canvas | Surface | Surface-muted |
|---|---:|---:|---:|
| `fg` | 18.36:1 | 19.64:1 | 17.58:1 |
| `fg-muted` | 5.64:1 | 6.03:1 | 5.40:1 |
| `fg-subtle` | 4.71:1 | 5.04:1 | 4.51:1 |
| `ink` | 17.62:1 | 18.85:1 | 16.87:1 |
| `accent` | 5.17:1 | 5.53:1 | 4.95:1 |
| `highlight` | 5.02:1 | 5.37:1 | 4.81:1 |
| `success` | 4.97:1 | 5.32:1 | 4.76:1 |
| `danger` | 5.29:1 | 5.66:1 | 5.07:1 |
| `border-control` | 3.41:1 | 3.65:1 | 3.27:1 |

Ink-fg on ink: **18.85:1**.

| Badge foreground | Its own 10% soft background |
|---|---:|
| `accent` | 4.81:1 |
| `highlight` | 4.68:1 |
| `success` | 4.64:1 |
| `danger` | 4.84:1 |

### Dark contrast

| Foreground | Canvas | Surface | Surface-muted |
|---|---:|---:|---:|
| `fg` | 17.58:1 | 16.44:1 | 15.17:1 |
| `fg-muted` | 7.66:1 | 7.17:1 | 6.62:1 |
| `fg-subtle` | 5.38:1 | 5.04:1 | 4.65:1 |
| `ink` | 17.58:1 | 16.44:1 | 15.17:1 |
| `accent` | 9.20:1 | 8.60:1 | 7.94:1 |
| `highlight` | 9.30:1 | 8.70:1 | 8.03:1 |
| `success` | 9.08:1 | 8.49:1 | 7.84:1 |
| `danger` | 7.28:1 | 6.81:1 | 6.28:1 |
| `border-control` | 3.73:1 | 3.49:1 | 3.22:1 |

Ink-fg on ink: **17.58:1**.

| Badge foreground | Its own 10% soft background |
|---|---:|
| `accent` | 7.31:1 |
| `highlight` | 7.37:1 |
| `success` | 7.22:1 |
| `danger` | 5.95:1 |

Neutral badge fg-muted on surface-muted is included above. Focus rings and accent selection bars use
the accent row; input/select/checkbox outlines use border-control. Button danger outlines use danger.
Border, border-strong, shadows and modal scrims are decorative only and have no foreground text pairing.

## Review

Run `npm.cmd run dev` from `app/` and open `http://127.0.0.1:1420/design`.
The unlisted gallery shows both themes simultaneously, every palette token, typography, corners,
elevation, all primitives, disabled/error/mixed states, dialogs and undo toasts.
Hover and keyboard focus can be inspected interactively.
The route rejects production access with 404 through its development-only load guard.
Review the shell at 1440 px and below 1100 px, then switch system/day/night and reload.
