# Intake Desk visual thesis

Direction: **dock-stamp constructivism**

Status: implemented for M1 and repair 1

Last updated: 2026-08-28

## Thesis

Intake Desk should feel like the one dependable sheet on a busy receiving
dock. Its visual language comes from manifests, routing marks, inspection
stamps, ruled count columns, and safety paint. Constructivist composition adds
bold hierarchy: status blocks interlock, labels align to a working grid, and
the current action interrupts the page in safety orange.

This fits the product because receiving work is physical, time-bound, and
evidence-led. A clear mark means more than a decorative illustration. The
interface must remain calm enough for repeated daily use and distinct enough
that a phone-sized screenshot cannot be mistaken for a generic SaaS dashboard.

“Dock-stamp” does not mean fake stains, shipping-container photos, hazard tape
everywhere, or nostalgia. “Constructivism” does not mean arbitrary diagonals.
Every rule, clipped corner, and block must explain sequence, grouping, or
status. Content and controls remain on solid plates.

## Stack and rendering decision

Use Svelte 5 with Vite and strict TypeScript. The receiving flow has reactive
counts, connection state, an offline outbox, scan capture, and route-level
state; Svelte expresses that without the runtime and ecosystem weight of
React. Rust/axum serves the API and the built static files from one Container
Apps origin. The web app must render usable HTML immediately and keep first
load under the budgets in the venture plan.

No runtime asset, font, icon, or script loads from a third-party CDN. The visual
system is CSS, text, and owned SVG first. Photography appears only when it is
customer evidence inside the product.

## Recognition cues

From a thumbnail, Intake Desk is recognized by four things:

1. A warm manifest-paper field with near-black ink.
2. One safety-orange action/status plate with a clipped lower-right corner.
3. Tall condensed headings aligned against ruled quantity columns.
4. A circular inspection stamp used only for finalized or resolved state.

Do not add gradient blobs, centered hero copy, floating glass cards, rounded
pill collections, stock warehouse photos, emoji, or generic outline icon sets.

## Color

### Day shift (default)

| Token | Value | Use | Verified contrast |
| --- | --- | --- | ---: |
| `--color-canvas` | `#F4E9CF` | Manifest background | — |
| `--color-surface` | `#FFF9E8` | Task plates and forms | — |
| `--color-surface-raised` | `#E7D9BB` | Selected ruled row | — |
| `--color-ink` | `#182327` | Primary text | 13.30:1 on canvas; 15.26:1 on surface |
| `--color-ink-muted` | `#526064` | Secondary text | 5.41:1 on canvas |
| `--color-safety` | `#B83A1B` | Primary action and discrepancy | 5.46:1 against surface |
| `--color-safety-ink` | `#FFF9E8` | Text on safety | 5.46:1 |
| `--color-route` | `#1F4B57` | Links, focus, active route | 9.07:1 against surface |
| `--color-success` | `#2E6A4F` | Matched/resolved mark | 6.07:1 against surface |
| `--color-warning` | `#8A5A00` | Pending/offline mark | 5.64:1 against surface |
| `--color-danger` | `#A52A2A` | Destructive/error mark | 6.74:1 against surface |
| `--color-rule` | `#A69B82` | Rules, not text | UI outline ≥3:1 where interactive |
| `--color-focus` | `#006C82` | Focus ring | 5.76:1 against surface |

### Night shift

Night is a real treatment for dark-preference users, not a color inversion.
Paper becomes a blue-black work surface and status inks become luminous chalk.

| Token | Value | Use | Verified contrast |
| --- | --- | --- | ---: |
| `--color-canvas` | `#11191C` | Night background | — |
| `--color-surface` | `#1B272A` | Night task plate | — |
| `--color-surface-raised` | `#26363A` | Selected row | — |
| `--color-ink` | `#F8EFD9` | Primary text | 15.55:1 on canvas; 13.38:1 on surface |
| `--color-ink-muted` | `#BEC7C3` | Secondary text | 10.30:1 on canvas |
| `--color-safety` | `#FF724C` | Primary action/discrepancy | 6.49:1 with `#151A1B` text |
| `--color-safety-ink` | `#151A1B` | Text on safety | 6.49:1 |
| `--color-route` | `#78B9C7` | Links and focus | 8.11:1 on canvas |
| `--color-success` | `#70C49A` | Matched/resolved | 8.52:1 on canvas |
| `--color-warning` | `#F3B84B` | Pending/offline | 9.98:1 on canvas |
| `--color-danger` | `#FF8074` | Destructive/error | 7.28:1 on canvas |
| `--color-rule` | `#56666A` | Rules | Non-text only |
| `--color-focus` | `#78B9C7` | Focus ring | 6.43:1+ on surfaces |

Never use color alone. Every state combines a word, a shape or icon, and
color: “Short · 2 each,” “Saved on this device,” or “Resolved.”

## Typography

Two self-hosted OFL families form the intended pairing:

- **Barlow Condensed** (600 and 700, Latin subset) for display headings,
  routing labels, buttons, and stamps. Its industrial proportions use narrow
  space without imitating a terminal.
- **Atkinson Hyperlegible Next** (400 and 700, Latin subset) for body, forms,
  long identifiers, help, and data. Its differentiated forms support hurried
  reading and low-vision users.

M1 must source WOFF2 files from the families' canonical repositories, retain
their OFL files under `public/fonts/`, record exact commit/download URLs below,
and keep the combined transfer ≤120 KB. Use `font-display: swap`; preload at
most the body regular and display semibold. Until those files are committed,
the scaffold uses honest system fallbacks and does not make missing font
requests.

Fallbacks:

```css
--font-display: "Barlow Condensed", "Arial Narrow", "Roboto Condensed", sans-serif;
--font-body: "Atkinson Hyperlegible Next", "Segoe UI", system-ui, sans-serif;
--font-mono: ui-monospace, "SFMono-Regular", Consolas, monospace;
```

Type sizes use `clamp()` only at the two display levels. Task body remains a
stable 16–18 px. Quantity columns and timestamps use `font-variant-numeric:
tabular-nums lining-nums`. Uppercase is limited to short route labels and
stamps with at least `0.08em` tracking; sentences stay sentence case.

| Token | Size / line height | Use |
| --- | --- | --- |
| `--text-xs` | 12 / 16 px | Build IDs and nonessential metadata only |
| `--text-sm` | 14 / 20 px | Labels; never the sole task instruction |
| `--text-body` | 16 / 24 px | Default controls and text |
| `--text-lead` | 20 / 28 px | Intro and important explanation |
| `--text-h3` | 28 / 32 px | Task section heading |
| `--text-h2` | `clamp(32px, 5vw, 40px)` / 1.05 | Major section heading |
| `--text-h1` | `clamp(40px, 8vw, 56px)` / 0.96 | One page headline |

Readable prose is 45–70 characters. A PO table may be wider because labels and
values form columns, not prose.

## Spacing, grid, and shape

The base unit is 4 px; common spacing stays on an 8 px rhythm.

```text
space: 4 · 8 · 12 · 16 · 24 · 32 · 48 · 64 · 96
radius: 0 · 2 · 6
rule: 1 px quiet · 2 px working · 4 px status
shadow: none by default · one hard 4 px offset for an open overlay
```

The content grid is mobile-first. At 390 px it uses 16 px gutters and one task
column. From 640 px it uses 24 px gutters and 12 columns. From 1024 px it uses
32 px gutters with a maximum working width of 1440 px. The landing composition
is intentionally left-weighted: headline occupies columns 1–7, a live manifest
strip occupies 8–12 and drops below on small screens. Product screens use a
3-column context rail only at desktop sizes.

Group by spacing before adding boxes. Rules belong to manifests and timelines;
they do not outline every section. Independent records may sit on solid plates.
Primary action plates use a CSS `clip-path` with an 8 px lower-right chamfer,
but retain a rectangular 44 px hit target and focus outline.

## Interaction grammar

- **Find/scan:** the capture field looks like a routing strip and remains first
  in the receive tab order. A recognized code moves focus to its quantity; an
  unknown code remains selected and explains how to find the line manually.
- **Count:** plus/minus controls move in the declared unit. Direct entry accepts
  decimal text. Expected, received, and difference remain adjacent.
- **Classify:** a mismatch opens its small condition set in place. No modal is
  required for ordinary classification.
- **Finalize:** the button names the result and counts unresolved lines. A
  specific confirmation names the PO and permanent effect.
- **Correct:** finalized data never re-enters edit mode. “Record a correction”
  opens a new form and shows the event it will supersede.
- **Navigate:** links are underlined or use a visible route arrow. Buttons are
  solid actions. Browser back restores route, focus, and scroll.

## Motion

Motion communicates physical origin and status only:

- `120 ms`: press, focus-adjacent color, small disclosure.
- `180 ms`: row insertion or connection-state change.
- `260 ms`: panel from its trigger, route content fade/translate ≤8 px.
- One signature motion: on successful finalization, the “Received” stamp moves
  down 6 px while fading in and makes a single 1-degree settle. It never loops.

Animate only transform and opacity. Nothing parallax-scrolls or auto-plays. A
change that affects quantities is announced in text before animation. Under
`prefers-reduced-motion: reduce`, durations become 1 ms, smooth scrolling is
disabled, the stamp appears in place, and no spatial movement remains.

## Component behavior

The implementation inventory is
[component-inventory.md](./component-inventory.md). Components use semantic
HTML first. Product components do not hide domain rules: for example,
`QuantityStepper` emits a decimal string and unit; the receipt domain service
decides whether it is valid.

Controls meet these shared states:

- Default, hover where supported, pressed, focus-visible, disabled, busy.
- Invalid with bound text and `aria-describedby`; error is announced once.
- Offline with the exact storage consequence, not just a cloud-slash icon.
- Empty with what appears here and one next action.
- Loading with reserved space and a text alternative after 2 seconds.

Buttons and inputs are at least 44 px high with 8 px between adjacent targets.
Icons that are not universal include labels. Destructive actions name the
record, require a specific confirmation, and offer undo when deletion is not
immediate.

## The five key screens

### 1. Landing and live sample

The header is a horizontal routing label, not a floating nav. A narrow
“RECEIVING / 01” label sits above the one plain H1. The sample action is the
only safety-orange block. Three facts sit as stamped text lines, not feature
cards. Beside or below, one live NB-1047 manifest strip already shows items and
an orange difference so the product, not an illustration, proves the idea.

Sections then follow the standard site order: live product, three verb steps,
limits/privacy, exact Dock price, footer. Section boundaries use alternating
rule direction and ample blank paper. The page never opens with a centered
logo or abstract hero.

### 2. Purchase-order inbox

The H1 is “Find the delivery.” A full-width scan/search strip comes first. PO
rows behave like ruled manifest entries: supplier and PO dominate; due date,
line count, and state align right. Groups are “Due,” “Part received,” and
“Closed.” Filters sit on a narrow vertical rail at desktop and a labelled sheet
on phone. The empty state says “No open purchase orders” and links to import.

### 3. Receive delivery

The PO reference and connection strip are always in view but not sticky over
content. Each line reads expected → received → difference. Received is the
largest editable number. A scanned line gets a 4 px route-blue rule; a mismatch
gets text and safety color. On phone, one line expands for counting while the
rest remain concise. The final action stays in normal flow with a safe-area
padding footer; it never covers the last line.

### 4. Discrepancy evidence

The H1 names the case and supplier. The first strip states exactly “2 each
short · 1 seal damaged.” Evidence thumbnails use fixed aspect ratios and
captions. The notification preview resembles a clean dispatch sheet, not an
email client. The immutable timeline runs down the outside edge with sequence,
actor, time, and correction links. Resolved state receives the circular stamp.

### 5. Import and administration

The mapping table is the main sheet: source header, target field, sample,
status, and correction share a row. Errors do not disappear when valid rows
are filtered. On narrow screens each source column becomes a definition list.
Settings have real URL tabs. Billing and retention state use plain sentences;
backup status never uses a green dot without a timestamp and word.

## Responsive intent

- **≤639 px:** show current task, scan/count controls, connection consequence,
  and primary action. Drop decorative index numbers, repeated descriptions,
  wide previews, and audit side rail. Convert tabular rows to labeled pairs.
- **640–1023 px:** use 5/7 manifest/action columns. Keep filters collapsed.
  Evidence uses two columns. The timeline follows content.
- **≥1024 px:** show context or audit rail. Keep line length capped; do not
  stretch controls across the viewport. Hover adds information but never
  reveals the only way to act.
- Respect `env(safe-area-inset-*)`. Test 320 px, 390 px, landscape, 200% zoom,
  virtual keyboard resize, and pointer coarse/fine.

## Accessibility

- Exactly one `<h1>` per route; ordered headings, header/nav/main/footer, skip
  link, visible link affordance, and a polite route live region.
- Route changes set title, move focus to H1, and restore scroll/focus on browser
  navigation. Dialogs name themselves, trap focus, close with Escape, and
  return focus to the invoking control.
- Scanner capture is an ordinary labelled input enhanced for burst timing. It
  has no global key listener while the user types elsewhere.
- Focus uses a 3 px ring at ≥3:1 and a 2 px offset. Touch targets are ≥44 px.
  Text survives 200% zoom without clipped actions or status.
- Text contrast is ≥4.5:1; large text and interactive outlines are ≥3:1. State
  never relies on color, position, texture, sound, or motion alone.
- Forms have persistent labels, described requirements and errors, and an
  error summary after failed finalize/import. Async status uses restrained live
  regions. Tabular data includes headers or labeled mobile pairs.
- Meaningful customer evidence gets a user caption; decorative rules and stamp
  fragments are hidden. Social art alt describes its purpose, not its pixels.

## Empty, loading, error, and offline language

Use these patterns, replacing identifiers precisely:

- Empty: “No open purchase orders. Import a CSV to add the next delivery.”
- Loading: “Loading open purchase orders…” followed after 8 seconds by “This is
  taking longer than usual. Retry without losing your draft.”
- Validation: “Received quantity must be zero or more. Enter the count in
  each.”
- Conflict: “NB-1047 changed after you opened it. Compare the new order before
  you finalize this receipt.”
- Offline draft: “Saved on this device. Keep this tab until it syncs.”
- Sync error: “The receipt is still on this device. Reconnect and retry.”
- Rate limit: “Too many attempts from this connection. Try again in 42
  seconds.”
- Storage full: “This photo was not saved. Free device storage or remove the
  photo, then try again.”

Errors state what happened and one next step. They do not use “Oops,” blame the
user, or expose implementation terms.

## Art and asset provenance

Only hand-made, generated-for-this-product, or direct product screenshots are
allowed. No stock art, copied icon packs, third-party logos, or text baked into
an image.

| Asset | Source / license | Status |
| --- | --- | --- |
| `public/favicon.svg` | Hand-authored geometric dock/receipt mark by Param Factory for this repository; covered by repository MIT license. | Shipped 2026-08-28 |
| Header wordmark | Live text plus the owned favicon geometry; no image wordmark. | Shipped M1 |
| Hero/live art | Real HTML demo manifest plus a hand-authored SVG evidence sketch; no raster hero. | Shipped M1 |
| `public/og-image.svg` | Hand-authored 1200×630 composition from the owned mark, manifest geometry, and token palette. | Shipped M1 |
| `public/apple-touch-icon.svg` / `.png` | Hand-authored and locally rasterized from the owned receipt geometry. | Shipped M1 |
| Product illustrations/icons | Small hand-authored SVG using 2 px square strokes; disclose author/date here as added. Prefer words when an icon is not universal. | Planned per milestone |
| Product screenshots | Captured from the deterministic demo; repository/product rights. | Planned M1+ |
| Barlow Condensed Semibold WOFF2 | SIL Open Font License 1.1; `jpt/barlow` commit `dc2940e2e04ef4ec96c07e23e0f02aefbddd343b`, path `fonts/woff2/BarlowCondensed-SemiBold.woff2`; license retained in `public/fonts/OFL-Barlow.txt`. | Shipped M1 |
| Atkinson Hyperlegible Next Regular/Bold WOFF2 | SIL Open Font License 1.1; `googlefonts/atkinson-hyperlegible-next` commit `7925f50f649b3813257faf2f4c0b381011f434f1`, paths under `fonts/webfonts/`; license retained in `public/fonts/OFL-Atkinson-Hyperlegible-Next.txt`. | Shipped M1 |

No generative model has been used for a shipped visual asset in this planning
work order. If a future builder generates one, add the tool/model, full prompt,
date, edits, license/usage basis, source file, and exported sizes to this table.
Generated imagery must be reviewed for text, symbols, seams, hands, and
unintended marks. It may not replace the live product preview.

## Content and metadata

First-screen copy and terminology are fixed in the venture plan. Titles follow
“Place — Intake Desk,” while home is “Intake Desk — check supplier deliveries.”
Every route owns a ≤155-character description, canonical URL, Open Graph and
Twitter metadata. The favicon, Apple icon, theme color, 404, robots, sitemap,
security headers, and build ID use this visual system.

The footer always includes: “Intake Desk records supplier deliveries and their
exceptions.”, Privacy, Terms, “Built by Param Factory,” and build ID. If any
generated art is later shipped, the footer or About disclosure says so.

## Performance rules

- Initial JS ≤200 KB gzip, target ≤150 KB; CSS ≤50 KB; fonts ≤120 KB; any
  mobile hero raster ≤300 KB. The current direction should need no hero raster.
- Self-hosted subsets with `font-display: swap`; preload no more than two.
- Responsive images reserve dimensions, use AVIF/WebP, and lazy-load below the
  fold. Customer evidence uses thumbnails before originals.
- Animate only opacity/transform; batch reads/writes; virtualize only after
  measuring large PO lists; use `content-visibility` below landing fold.
- Service worker precaches versioned shell and demo seed. Hashed assets get
  immutable cache headers; HTML and the worker do not.

## Design acceptance test

M1 review captures 390×844 and 1440×900 screenshots of every key route in day
and night treatment. In two seconds, five evaluators must identify the current
record, its state, and the next action. Review also checks grayscale, common
color-vision simulations, keyboard-only, screen-reader smoke, 200% zoom,
reduced motion, no-texture mode, camera denial, offline, and storage error. Any
decorative element that competes with expected/received counts is removed.
