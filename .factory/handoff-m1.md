# M1 handoff — see and complete the dock job

Work order: `venture-purchase-intake-desk-m1`

Date: 2026-08-28

Status: **BUILT, DEPLOYED, AND READY FOR INDEPENDENT REVIEW/POLISH**

Production: <https://purchase-intake-desk.sociobot.in>

Demo: <https://purchase-intake-desk.sociobot.in/?demo=1>

## What shipped

- The standard landing sequence with the fixed plain-language first screen,
  live NB-1047 manifest, three-step explanation, scope boundary, exact Dock
  price, and honest “Accounts open in the next milestone” state.
- All planned public/demo routes, distinct titles and descriptions, History API
  navigation, route announcements/focus, a styled 404, legal pages, sitemap,
  robots, PWA manifest, icons, social art, and security headers.
- The complete West Yard / Northline Bearings demo in IndexedDB database
  `intake-desk:demo:v1`: PO review, typed or keyboard-scanner line lookup,
  exact decimal count handling, damage condition, evidence fixture,
  finalization, combined discrepancy record, hash-linked correction events,
  deterministic reset, and versioned receipt CSV.
- A versioned service worker that keeps the shell and current demo usable after
  a network drop. Demo requests are same-origin only. It has no auth, billing,
  analytics, email, AI, or production data adapter.
- Dock-stamp constructivism across day/night colors and phone/desktop layouts,
  using self-hosted OFL Barlow Condensed and Atkinson Hyperlegible Next fonts.
  The original mark, social image, and evidence sketch are hand-authored.
- Rust/axum static delivery with build SHA health output, graceful shutdown,
  security headers, and a `tower_governor` 20 request/second, burst-40 limit
  keyed from the first valid `X-Forwarded-For` address. `/health` is exempt.

## Milestone boundary decision

The work-order boilerplate mentions auth, server persistence/migrations, and
billing. The source-of-truth plan explicitly assigns them to M2 and says M1
must work without an account or server write and must not expose fake sign-in
or checkout. This implementation follows the named M1 contract. No plan change
was needed. The M1 persistence is real, isolated IndexedDB persistence; the
durable tenant database, Entra CIAM, and Dodo-backed Sociobot billing remain M2.

## Verification evidence

Clean-clone commands:

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
```

Results:

- Svelte/TypeScript: 0 errors and 0 warnings.
- Vitest: 7 files, 11 tests passed.
- Rust: 3 API integration tests passed; unit/doc targets passed; clippy passed
  with warnings denied; rustfmt passed.
- Playwright: 22/22 passed. Each of nine `@claim:*` declarations passed in a
  fresh desktop context and a Chromium 390×844 mobile context. Two additional
  accessibility/navigation tests passed in both projects.
- Axe: zero serious/critical findings across `/`, every public/demo route,
  legal routes, and `/404` covered by the route scan.
- Production bundle: 28.28 KB gzip JS, 4.83 KB gzip CSS, 111 KB WOFF2 fonts,
  no hero raster. Required `dist/` files were asserted after build.
- Lighthouse mobile: performance 99, accessibility 100, best practices 100,
  SEO 100; LCP 1,956 ms; CLS 0; total blocking time 43 ms.
- Visual checks: 390×844 and 1440×900 landing/receive captures have one H1,
  no horizontal overflow, visible task state, and a reachable primary action.
- Live cold check: HTTP 200, 637 ms navigation, title `Demo — Intake Desk`,
  `lang=en`, one H1, one main, no missing alt text, no unlabeled buttons, and
  no page/console errors. Evidence is under `.factory/evidence/m1-live/`.
- ACR built the multi-stage image and Container Apps served it non-root on
  `PORT=8080`. `/health` reports the deployed source SHA.

## Claim coverage

`.factory/claims.json` contains the nine M1 claims. Each has exactly one named
Playwright declaration in `tests/e2e/claims.spec.ts`, starts from `/?demo=1`,
and observes the promised result rather than control presence. The offline and
isolation tests use a fresh browser context and no external service.

## Known gaps and operator action

- Independent review and polish have not run yet, so the plan correctly leaves
  M2 blocked despite this build being complete.
- Before M2 verification, register
  `https://purchase-intake-desk.sociobot.in/auth/callback` on Entra SPA client
  `25c704f4-465a-47af-80ab-2c489466b697`.
- Before M2, register the recurring test product in the Sociobot billing engine
  at $149 USD per site each month and mount durable `/data` plus a backup
  target. These actions do not belong in M1.
- Camera decoding is deliberately not claimed in M1. The UI verifies camera
  permission/unsupported states and keeps typed/scanner entry available.

## What M2 needs

Implement the planned Entra PKCE/JWKS path, reversible SQLite migrations,
tenant/site/membership authorization, durable receipt APIs, production
IndexedDB outbox, idempotency and conflict handling, encrypted entitlement
attachment, pilot Sociobot checkout/verification, `/ready`, and role/isolation
tests. Keep the M1 demo namespace and its nine claim tests unchanged.

## Repair 4 addendum — 2026-09-05

The M1 demo remains the current public milestone. Repair 4 did not add a new
product capability. It fixed the durable service readiness path used by the
already-present server work:

- Fresh `/data` now creates and checks
  `intake-desk-rollback.sqlite3`, the rollback-journal database actually used
  by the one-replica Azure Files deployment.
- A temporary isolated-volume regression proves initial readiness, evidence
  directory detection, shutdown/reopen, and persisted data.
- The demo reset/workspace boundary now has an explicit outcome claim.
- The hosted CIAM session claim is declared, but it is intentionally skipped
  until an operator supplies a dedicated product account and confirms the
  callback registration. This is an external dependency, not a completed M1
  claim.

Implementation: `64df103df5f62e428da00cd4e25d6e3f47e6cae1`. Documentation:
`ae9873e66ea093f3aa2bf58355fe6e352dbfcb53`. The deployed image is immutable
digest `sha256:3fb074577030ac81a821290037a004738e4c02a2935a6d53bb2c36f84382c293`.
