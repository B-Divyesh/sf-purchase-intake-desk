# Planner handoff

Work order: `venture-purchase-intake-desk-plan`

Role: planner

Date: 2026-08-28

State: complete; M1 product work has not started

## What was done

- Wrote `.factory/plan.md` as the venture execution contract: PRD, evidence,
  wedge, monetization, architecture, tenancy/data model, auth, billing, offline
  sync, files, AI boundary, rate limits, operations, M1–M5, tests/DoD, and a
  risk experiment for every material unknown.
- Wrote `.factory/design.md` for dock-stamp constructivism with verified day and
  night palettes, typography, spacing, shape, interaction, motion, key screens,
  accessibility, responsive behavior, performance, and asset provenance.
- Wrote `.factory/claims.json` with nine M1 claims and one required Playwright
  tag/observable sandbox procedure for each. No skipped/fake tests were added;
  M1 must implement them with the product behavior.
- Wrote `.factory/demo.md` with the `/?demo=1` entry, deterministic NB-1047
  sample, `intake-desk:demo:v1` isolation, reset, and offline verification.
- Wrote `.factory/component-inventory.md` with 20 components, states, keyboard
  behavior, phone behavior, composition rules, and stable fixture names.
- Updated the researched brief to its supplied `ADMITTED` state and timestamp.
- Scaffolded Svelte 5 + Vite + strict TypeScript, the source design tokens,
  typed component inventory, and a semantic planning shell that clearly says
  the product is not built.
- Scaffolded Rust 2021 + axum with static serving, `/health`, build identity,
  security headers, structured JSON logs, default `PORT=8080`, and graceful
  shutdown. Product APIs and persistence correctly remain M2 work.
- Added exact dependency locks, Vitest/API tests, Playwright 1.58.2 config for
  future claim tests, GitHub Actions checks/tests/build, `.dockerignore`, and a
  multi-stage non-root Dockerfile that does not use `.git`.
- Rewrote README with product/customer context, honest planning status,
  run/test/build/container instructions, and operator boundaries. Existing MIT
  license was retained.

## How it was verified

From `/work/repo`:

```sh
npm ci
npm run check
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
npm test
npm run build
```

Results:

- npm audit: 0 vulnerabilities.
- Svelte/TypeScript: 0 errors and 0 warnings.
- Rust fmt and clippy: pass with warnings denied.
- Vitest: 2 files, 4 tests passed.
- Rust: health integration test passed; unit/doc targets passed.
- Vite and release Rust build passed; `dist/index.html` exists.
- Initial planning shell: 12.5 KB JS gzip and 1.9 KB CSS gzip.
- Axum smoke: `/health` returned 200 with `{"status":"ok","build_sha":"dev"}`
  and CSP, nosniff, and referrer headers.
- Chromium at 390×844: correct title and `lang`, one H1, one main landmark, no
  horizontal overflow, and no console/page errors.
- SIGINT produced the graceful shutdown log and exited cleanly.
- Claims JSON parses, contains nine unique IDs, and every test command uses its
  matching `@claim:<id>` tag.

Docker itself was not available in this worker image, so the Dockerfile was
reviewed but not executed locally. CI performs source builds but intentionally
does not publish or deploy.

## Known gaps (intentional)

- No product workflow, routing, IndexedDB, service worker, demo, legal routes,
  accounts, billing, database, uploads, email, analytics, or AI is implemented.
  This is a planning work order, and the visible shell says so.
- M1 claim commands will have no implementation until the M1 builder adds one
  tagged test per claim. The placeholder test directory explains this rather
  than creating misleading passing tests.
- Product fonts, Open Graph/Apple art, sitemap, robots, CSP for later external
  connections, and real `/privacy`, `/terms`, and `/404` pages are M1 scope.
- PostgreSQL was not selected for launch. The plan uses SQLite WAL on persistent
  `/data` to satisfy the no-required-env runtime contract, with an evidence
  threshold for a later PostgreSQL migration.

## Needs operator action

These are not needed for M1 and must not be performed from this repository:

1. Before M2 production review, register
   `https://purchase-intake-desk.sociobot.in/auth/callback` on Entra SPA client
   `25c704f4-465a-47af-80ab-2c489466b697`.
2. Register `purchase-intake-desk` in the Sociobot billing engine as the Dock
   recurring subscription at $149 USD/site/month, including a pilot/test
   product and return URL.
3. Before production persistence, mount durable `/data` and a separate backup
   target; confirm restore ownership and alert routing.
4. Configure transactional email only when M3 is ready to test real supplier
   notices. Configure the optional Sociobot gateway key only in M5.

## Next builder: M1

Read `.factory/plan.md`, `.factory/design.md`, `.factory/demo.md`,
`.factory/claims.json`, and `.factory/component-inventory.md` in full. Build
only “M1 — See and complete the dock job in a sandbox.” Preserve the fixed
West Yard / Northline Bearings NB-1047 sample, demo namespace, first-screen
copy, token names, and nine claim IDs. Finish the copy audit, real claim tests,
all public/legal routes, offline demo, accessibility/performance verification,
and `.factory/handoff-m1.md`. Do not start auth, production storage, billing,
supplier sending, or runtime AI before M1 review and polish pass.
