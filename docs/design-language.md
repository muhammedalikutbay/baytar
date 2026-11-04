# Baytar design language

## Principles and roles

Baytar is a calm, modern desktop workspace for independent veterinarians and small clinics.
White and near-black are the base. Clear headings, compact lists, soft corners and quiet hairlines
make daily clinical work easy to scan. All fonts and artwork are bundled for offline use.

- Turquoise guides clinical work: filled primary buttons, links, focus rings, active navigation,
  selected tabs, today markers and clinical icon chips.
- Orange calls for attention: overdue and due items, debt, expiry and stock warnings, and one dashboard stat.
- Secondary buttons use a solid surface and a control boundary; ghost buttons keep their surroundings.
- Success and danger pair explicit labels with small icons or badges. Never communicate status by color alone.
- Cards, fields, sidebar and dialogs stay solid neutral surfaces. Large saturated fills are reserved for
  primary buttons and the logo tile. Do not tint whole pages, forms or warning panels.

The canonical palette lives in app/src/app.css, using Tailwind v4 @theme static. The default Tailwind
palette is removed. Components use semantic utilities or variables. Only the canonical brand SVG and
its inline Logo component use literal brand colors, so the same artwork appears in both themes.

## Semantic tokens

| Token | Light | Dark | Role |
|---|---|---|---|
| --color-canvas | #F7F7F9 | #0B0B0F | Main canvas |
| --color-surface | #FFFFFF | #141418 | Solid cards, fields, sidebar and title bar |
| --color-surface-muted | #F2F2F5 | #1C1C22 | Neutral hover, disabled and selected controls |
| --color-border | #E9E9EE | #26262E | Decorative hairlines and grid |
| --color-border-strong | #C9C9D2 | #3A3A44 | Stronger decorative lines |
| --color-border-control | #85858F | #6B6B76 | Essential boundaries, minimum 3:1 |
| --color-fg | #0B0B0F | #F2F2F5 | Headings and body text |
| --color-fg-muted | #62626C | #A1A1AA | Supporting text |
| --color-fg-subtle | #6E6E78 | #858590 | Auxiliary text on solid neutral surfaces |
| --color-ink | #111114 | #F2F2F5 | Neutral checked marks and selection |
| --color-ink-fg | #FFFFFF | #0B0B0F | Text on ink |
| --color-primary | #11756F | #4FC3B8 | Filled primary buttons |
| --color-primary-fg | #FFFFFF | #0B0B0F | Text and icons on primary buttons |
| --color-primary-hover | #0F665F | #70D3C9 | Primary button hover |
| --color-accent | #11756F | #4FC3B8 | Clinical guidance, links and focus |
| --color-highlight | #A9521C | #F0A06A | Attention, overdue, debt and stock |
| --color-success | #1F7A4D | #5BC48A | Success text and icons |
| --color-danger | #C42B1C | #F27A6E | Errors and destructive outlines |
| --color-window-close | #C42B1C | #C42B1C | Close button hover |
| --color-window-close-fg | #FFFFFF | #FFFFFF | Close hover icon |

Soft tokens for accent, highlight, success and danger are opaque 10% sRGB mixes with surface:
color-mix(in srgb, var(--color-ROLE) 10%, var(--color-surface)). Use them for icon circles, badges,
active nav and compact card header strips. The background does not depend on the canvas beneath.
Neutral chips use surface-muted with fg-muted. The modal scrim is rgb(11 11 15 / .42).

## Typography

Bundle @fontsource-variable/plus-jakarta-sans/wght.css with Latin and Latin Extended subsets, including
Turkish. The license is app/static/fonts/Plus-Jakarta-Sans-OFL.txt. No remote font requests.
Fallbacks: ui-sans-serif, system-ui, sans-serif. Body uses 400; labels and actions 500; headings 600-700.

| Utility | Size | Line height | Weight | Tracking |
|---|---:|---:|---:|---:|
| type-page | 28 px | 1.25 | 600 | -0.035 em |
| type-title | 20 px | 1.35 | 600 | -0.025 em |
| type-section | 15 px | 1.5 | 600 | -0.01 em |
| type-body | 14 px | 1.55 | 400 | normal |
| type-caption | 12 px | 1.5 | 500 | normal |

The tabular utility enables font-feature-settings: "tnum" and tabular-nums. Apply it to stats, money,
doses, dates and aligned numeric columns. Tables enable it by default, as does the existing tabular-nums
class. Use fg-muted for supporting copy over the patterned canvas; fg-subtle is for solid neutral surfaces.

## Canvas, hierarchy and components

Only the main content area has a 32 px grid at 45% of the border token, fading through a radial mask
from the top. Two blurred radial glows sit behind it: turquoise top-left and orange top-right, with
opacity .10 in light mode and .16 in dark mode (below the .12/.18 caps). They never affect card opacity.
Use no other decorative gradients or animation. The brand tile is the other gradient exception.

PageHeader aligns a title, one-line subtitle and optional clinical icon chip with the page's primary
action on the right. Actions wrap on narrow widths. Card supports an icon chip, heading, optional count,
and an actions snippet for a see-all link. StatTile has a large tabular value, label, small hint and icon chip.
IconChip has accent, highlight, neutral, danger and success tones. All are shown in /design in both themes.

Buttons have primary, secondary, ghost and destructive variants; danger remains an alias for existing callers.
Default form controls are 40 px high; default buttons are 44 px and small buttons are 36 px. Controls retain explicit focus-visible
outlines. Disabled buttons use surface-muted and fg-subtle. No low-opacity text.

Use a 4 px spacing base, with 6 and 10 px micro gaps. Page groups are 32 px apart; card padding is
24 px (20 px in narrower contexts). Border radii: sm 10, md 14, lg 20, xl 28 px and pill 9999 px.
Use hairline shadows for tiny raised controls, float shadows for dialogs and popovers, never heavy card shadows.

## Shell, brand and theme

The custom title bar is 40 px tall and spans the whole window. It contains the 24 px logo and Baytar,
centered global search (up to 560 px) with Ctrl+K, and three 46 px wide window controls. Close has
a red hover fill with a white icon. Drag regions exclude inputs and buttons; double-click toggles maximize.
Browser preview shows the controls without native actions. Maximize/restore tracks native resize events.
On macOS native traffic lights remain, custom buttons hide, and the bar reserves 78 px on the left.
Windows/Linux use no native decorations and retain a window shadow.

The sidebar is 232 px, or a 76 px icon rail below 1100 px. Workspace and clinical group labels separate
navigation. Active items use turquoise soft fill, a turquoise icon/text and a 3 px left bar; aria-current
also communicates selection. The main content has 32 px gutters and a maximum width of 1264 px including
gutters (24 px gutters at smaller widths, 16 px on narrow windows).

A 28 px high sun/moon segmented toggle sits beside the version at the bottom. With no stored preference,
the app follows prefers-color-scheme live. Choosing light/dark persists that choice. The settings link
clears the preference and immediately returns to live system theme tracking.

app/src-tauri/icons/app-icon.svg is the 1024 px brand master: rounded-square turquoise gradient
#2DD4BF to #0F766E at 135 degrees, white paw, small orange plus #F59E0B to #EA580C inside the main pad.
It contains no text. Logo.svelte uses the same shapes with unique gradient IDs at any size (20/28/40).
The favicon and all Tauri icons derive from the same master; regenerate with npx.cmd tauri icon
src-tauri/icons/app-icon.svg from app/. Species and empty-state paw icons remain ordinary clinical symbols.

## Accessibility and copy

Normal text must pass 4.5:1. Essential control boundaries and icons need 3:1. Decorative hairlines,
canvas grid, glows and logo artwork are exempt. Keep focus rings unclipped, support keyboard navigation,
label all icon buttons, preserve native form semantics and modal focus containment, and honor reduced motion.
The compact theme segments have 24 px wide targets; title-bar controls are 46 by 40 px.

UI copy lives in app/src/lib/i18n/tr.ts. Use Turkish sentence case and Turkish-aware date/number formatting.
Identifiers, comments and docs remain English. Never add real clinical data to examples or logs.

## Measured contrast reference

Computed using WCAG sRGB relative luminance: linearize each channel, apply
0.2126 R + 0.7152 G + 0.0722 B, then (lighter + .05) / (darker + .05).
Ratios below are rounded to two decimals after calculating with full precision.
Columns cover normal, hover, selected, disabled, placeholder and neutral badge surfaces.
Ink on surfaces also covers checkbox checks; ink-fg is used only on ink.
Soft pairs use the exact 10% sRGB mix, not an alpha estimate.

### Filled button contrast

| Pair | Light | Dark |
|---|---:|---:|
| primary-fg on primary | 5.53:1 | 9.20:1 |
| primary-fg on primary-hover | 6.80:1 | 11.09:1 |
| window-close-fg on window-close | 5.66:1 | 5.66:1 |

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
Review the shell at 1440 px and below 1100 px, then switch light/dark and use the settings reset to verify live OS theme changes.

## Form controls and selection

Labels use text-sm font-medium text-fg with a 6 px gap. Controls use 14 px text,
40 px minimum height and 12 px horizontal padding. Hints and errors remain type-caption.
Textareas begin at three rows. Optional leading lucide icons are 16 px, muted, and decorative;
PhoneInput retains its country button and EmailInput defaults to Mail. Date inputs retain native icons.
Required labels show a danger asterisk hidden from assistive technology; native required semantics remain.
Place RequiredNote once at the top of forms containing required fields.

Combobox binds the item value while showing its label, filters Turkish-folded labels and hints,
and supports keyboard selection, grouped results, optional custom text and clearing. TimeInput binds
HH:MM or an empty string, normalizes shorthand on blur and offers quarter-hour choices.
RecordPicker binds owner/patient records and delegates creation to the parent. Small directories start
expanded; larger directories start collapsed, and searches expand matches with all their pets.
