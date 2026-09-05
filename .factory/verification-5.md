# Verify supplier delivery intake — independent verification 5

**Verdict: FAIL — 1 high-severity finding and 1 untested claim.**

Verified on 2026-09-05 against:

- Implementation candidate: `64df103df5f62e428da00cd4e25d6e3f47e6cae1`
- Documentation candidate: `ae9873e66ea093f3aa2bf58355fe6e352dbfcb53`
- Starting checkout: `503d8d967e64768f8e0dfea32c5b291080358f99`
  (later report-only commit; no product-source differences from `64df103`)
- Live URL: <https://purchase-intake-desk.sociobot.in>
- Controller stage: `m1-building`

No product code was changed during this verification. The current M1 demo and
local receiving workspace pass. The repaired server readiness, tenant,
retention, evidence, entitlement, and rate-limit behavior also pass. Acceptance
still fails because the public hosted sign-in/session/sign-out claim cannot be
tested without the named external CIAM inputs.

## Finding

### High — hosted Dock sign-in remains untested

The landing page exposes **Sign in**. The README says an entitled Dock site uses
Sociobot Entra, and `.factory/claims.json` now correctly declares
`hosted-entra-session`. Its observable outcome is still untested.

- `ENTRA_E2E_USERNAME` and `ENTRA_E2E_PASSWORD` were absent.
- Callback registration for
  `https://purchase-intake-desk.sociobot.in/auth/callback` was not confirmed.
- The direct Chromium claim command ran and reported one skipped test.
- The full local and live suites each skipped the hosted claim.
- A fresh browser did prove the unauthenticated redirect starts at the expected
  Sociobot CIAM tenant with the expected public client ID, callback, scopes,
  authorization-code response, state, and S256 PKCE challenge.

This is also an external dependency. It is not evidence of a code defect, but
the work order permits PASS only with zero untested claims. An operator must
provide an isolated product test account, confirm the callback registration,
and run the documented claim command. No shared credential was used.

## First screen and core job

Fresh 1440×900 desktop and 390×844 phone contexts showed all three required
answers before scrolling:

- Job: **Check deliveries against the purchase order.**
- Audience: **For small receiving teams that need a clear record before the
  supplier van leaves.**
- First action: **Try it with sample data.** The adjacent note says it opens one
  ready PO without an account.

One click opened a populated Northline Bearings `NB-1047` sample. The count
screen showed 120 bearings matched, 46 of 48 belts with 2 short, and 24 seals
with 1 damaged. The persistent banner said **Demo — sample data, nothing is
saved** and kept **Reset demo** and **Start for real** available.

Changing the belt count to 41 and resetting restored 46. Fresh demo contexts
opened only `intake-desk:demo:v1`, made no `/api` requests, contacted no other
origin, and created no workspace database. The live
`demo-reset-preserves-workspace` claim separately imported `PO-402`, reset the
sample, then reopened the unchanged 30-each workspace order. No authenticated
or production record was read or written.

The finalized sample produced one discrepancy containing both the 2-each
shortage and 1-each damage, a three-line CSV plus header, and a correction event
without changing the original event hash.

Evidence:

- `evidence-verification-5/first-screen-desktop.png`
- `evidence-verification-5/first-screen-phone.png`
- `evidence-verification-5/demo-desktop.png`
- `evidence-verification-5/demo-phone.png`
- `evidence-verification-5/live-browser.json`

## Declared claims

All 23 claim selectors were invoked from the clean checkout. The 22 claims that
do not require an external identity passed. The hosted claim ran without the
unavailable inputs and skipped; it is not counted as passed.

| Claim | Result |
| --- | --- |
| `demo-entry-reset` | PASS — desktop and phone, local and live |
| `expected-vs-received` | PASS — desktop and phone, local and live |
| `exact-unit-conversion` | PASS — desktop and phone, local and live |
| `partial-discrepancy` | PASS — desktop and phone, local and live |
| `immutable-correction` | PASS — desktop and phone, local and live |
| `receipt-csv-export` | PASS — desktop and phone, local and live |
| `offline-reload` | PASS — desktop and phone, local and live |
| `demo-isolation` | PASS — desktop and phone, local and live |
| `scanner-manual-fallback` | PASS — desktop and phone, local and live |
| `current-count-finalization` | PASS — desktop and phone, local and live |
| `exact-decimal-export` | PASS — desktop and phone, local and live |
| `damage-count-validation` | PASS — desktop and phone, local and live |
| `real-po-workspace` | PASS — desktop and phone, local and live |
| `multi-po-retention` | PASS — desktop and phone, local and live |
| `demo-reset-preserves-workspace` | PASS — desktop and phone, local and live |
| `hosted-entra-session` | **UNTESTED — skipped; external account and callback confirmation absent** |
| `checkout-unavailable` | PASS — desktop and phone, local and live |
| `phone-qr-scanning` | PASS — recorded camera/decoder fixture, desktop and phone |
| `server-tenant-isolation` | PASS — exact Rust claim test |
| `server-record-reload` | PASS — exact Rust claim test |
| `server-audit-retention` | PASS — exact Rust claim test |
| `entitlement-read-only` | PASS — exact Rust claim test |
| `authenticated-cache-bypass` | PASS — desktop and phone, local and live |

Every claim ID occurs once in `.factory/claims.json` and once as a test tag.
The landing copy and README expose no additional current user capability that
lacks outcome coverage. Operational `/health` and `/ready` statements are also
covered by the integration suite and direct live checks.

Claim result: **22 passed; 1 untested.**

## Normal, invalid, boundary, and recovery paths

- A normal customer CSV imported, converted 2.5 cases × 20 to exactly 50 each,
  accepted correctly signed local evidence, finalized, exported, and survived
  later PO imports.
- A zero ordered quantity was rejected with a focused error that names the row
  and required correction.
- A file named `fake.png` with non-PNG content was rejected.
- `UNKNOWN-99` produced a specific not-on-PO message and kept the manual line
  fallback available.
- Damaged quantity greater than received quantity could not finalize.
- Offline reload retained changed demo counts; reconnect sent no demo data.
- Reset restored the deterministic sample without changing the separate local
  workspace.
- A fresh temporary server directory became ready, created
  `intake-desk-rollback.sqlite3`, shut down cleanly, restarted on the same
  directory, and returned ready again. The record-reopen and audit-retention
  tests prove server records and evidence survive database reopen.

## Accessibility, responsive behavior, privacy, links, and 404

- Live route scans covered `/`, `/demo`, purchase-order and receiving routes,
  `/start`, `/app`, `/privacy`, `/terms`, `/404`, and an unknown route.
- Every checked route had one H1, one main landmark, one description, one
  canonical link, ordered usable content, no horizontal overflow, and zero Axe
  violations in the regular suite.
- A separate dark, 390 px, reduced-motion scan covered home, demo, receiving,
  privacy, terms, and an unknown route: zero Axe violations, zero overflow, and
  zero running animations.
- The 320 px receiving screen and the 200% text-size check had no horizontal
  overflow; the demo banner and final action remained available.
- The first Tab focused the visible skip link; Enter moved focus to `main`.
  Route changes and dialogs passed focus-return tests. Controls passed the 44 px
  target checks.
- Demo and real-workspace browser tests made only same-origin requests. No
  analytics, advertising, AI, third-party font, or third-party script request
  appeared. Authenticated API responses bypass the shared cache.
- Internal links returned 200 except `/404`, which deliberately returned 404.
  Privacy and support links are explicit `mailto:` links. The external
  Sociobot/Param Factory link was not fetched because this work order forbids
  connecting to the shared Sociobot service.
- `/404` and a new unknown URL returned the designed page with HTTP 404, its own
  title, one H1, a main landmark, zero Axe violations, and routes home/sample.
  The expected failed-navigation console entry for a 404 is not a defect.

Evidence: `evidence-verification-5/boundary-accessibility.json`,
`dark-reduced-phone.png`, `receive-320.png`, and `text-200.png`.

## Backend, rate limits, and live identity

- Live `/health` and `/ready` returned 200.
- A 120-request burst to protected `/api/v1/me` from one forwarded client
  produced 96×401 and 24×429. The 429 response included `Retry-After: 0`.
  After two seconds, the client was admitted and received the expected 401 with
  `WWW-Authenticate: Bearer`.
- API responses are `no-store`. Live responses include CSP, HSTS, `nosniff`, a
  strict referrer policy, and a camera-only permissions policy.
- Exact temporary-SQLite tests passed for same-ID tenant isolation, assigned
  member reload, request-time audit data, append-only corrections, evidence
  retrieval, consistent backup, and read-only lapsed entitlement behavior.

The public endpoints currently report build
`503d8d967e64768f8e0dfea32c5b291080358f99`, not the `64df103` stamp stated in
the incoming handoff. This is not classified as a product defect:

- `git diff 64df103..503d8d9` contains only README and `.factory` files.
- There are no source, API, test, package, public-asset, or Dockerfile changes.
- The live main bundle is byte-for-byte equal to a `64df103` candidate build
  after normalizing only the displayed 12-character build stamp and generated
  source-map filename.
- Live readiness and the clean-volume regression prove the `64df103` repair is
  present. The later report-only stamp does not require a new product image.

Implementation reviewed remains `64df103`; documentation repaired at
`ae9873e`; verification began from report-only `503d8d9`.

## Performance and clean-checkout gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 86 packages, 0 vulnerabilities |
| `npm run check` | PASS — 0 errors, 0 warnings |
| `npm test` | PASS — 17 web tests and 14 Rust tests |
| Every declared claim command | 22 PASS; hosted claim SKIPPED/UNTESTED |
| `npm run test:e2e` | PASS — 44 passed, 2 hosted skips |
| `npm run build` | PASS — `dist/` and release binary produced |
| `cargo fmt --manifest-path api/Cargo.toml -- --check` | PASS |
| `cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings` | PASS |
| `/opt/fleet/lib/verify-url.sh` | PASS — 707 ms, no console errors |

The live full browser suite ended successfully with 42 direct passes, 2 hosted
skips, and 2 service-worker checks that timed out once during the concurrent
run and passed on their configured retry. A dedicated one-worker live rerun
then passed the service-worker update check 10/10 times; the offline claim
itself passed without retry on desktop and phone.

Fresh live Lighthouse scores were performance 98, accessibility 100, best
practices 100, and SEO 100. LCP was 2.14 s, CLS 0, and total blocking time
29 ms. The initial app JS is 104.59 KB raw / 34.99 KB gzip; CSS is 22.08 KB raw
/ 5.04 KB gzip; self-hosted WOFF2 fonts total 111.33 KB. The 240.96 KB raw CIAM
chunk is deferred until sign-in.

## Earlier findings

| Earlier finding, including minor issues | Current disposition |
| --- | --- |
| Final records ignored entered counts | Fixed; current-count finalization passes locally and live. |
| Candidate could not do a real PO/evidence job | Fixed for the current local workspace; import, evidence, finalization, export, and retention pass. |
| Decimal CSV values were inexact | Fixed; `46.1 - 48` exports as `-1.9`. |
| Impossible damaged quantities finalized | Fixed; finalization is blocked with focused guidance. |
| Claim inventory omitted product outcomes | Fixed for current outcomes; the hosted claim is now declared but remains untested in the finding above. |
| Touch targets were below 44 px | Fixed; desktop and phone target scans pass. |
| Production caching policy was missing | Fixed; hashed assets are immutable and HTML/service worker are no-store. |
| Unknown routes returned 200 | Fixed; designed unknown routes return HTTP 404. |
| Docker pinned a forbidden Rust minor | Fixed; it uses `rust:1-slim`. |
| Service-worker cache used a milestone name | Fixed; build-versioned cache checks pass. |
| A second PO destroyed the first receipt/evidence | Fixed; multi-PO retention passes. |
| No shared durable backend existed | Repaired core tenant SQLite paths pass; hosted user access remains the one untested dependency. |
| Phone QR scanning was missing | Fixed for the declared recorded camera fixture; typed/scanner fallback also passes. |
| Dock price/checkout was broken or unclaimed | Fixed for M1: $149 is explicitly planned and checkout is accurately unavailable, with no payment link. |
| Tenant IDs could overwrite another tenant | Fixed; exact same-ID tenant isolation passes. |
| Assigned users could not reload records | Fixed; member and record reopen test passes. |
| Audit timestamps, corrections, evidence, and backup were unsafe | Fixed; exact audit-retention test passes. |
| Entitlement status was not enforced | Fixed; writes return 402 while reads/export remain available. |
| Authenticated responses could enter shared cache | Fixed; cache-poisoning outcome test passes. |
| 401 lacked a Bearer challenge / auth verifier behavior was incomplete | Fixed in API and verifier tests; complete hosted session remains untested. |
| Zero quantities imported | Fixed; live invalid import is rejected. |
| Controlled navigation masked real 404 responses | Fixed; live unknown navigation returns 404. |
| Route metadata was duplicated | Fixed; one description and canonical link per checked route. |
| Landing preview nested a complementary landmark | Fixed; Axe finds no violation. |
| Fresh durable volume never became ready | Fixed; clean process and integration regression both return ready before and after restart. |
| README named the wrong database and WAL mode | Fixed; it names the rollback database and `DELETE` journal. |
| Demo reset/workspace separation lacked its own claim | Fixed; the explicit claim passes locally and live. |
| Hosted sign-in was absent from the inventory | Declaration fixed; observable hosted outcome remains untested and is the current finding. |

## Milestone and external dependencies

Controller stage `m1-building`: the shipped M1 public demo and local receiving
workflow pass. Repaired M2 server primitives were tested because the current
site and README expose them. M3 supplier email, accounting/ERP sync,
notifications, and later operations work remain planned and were not demanded
or described as shipped.

External dependencies are separate:

1. **Acceptance blocker:** isolated CIAM credentials and callback-registration
   confirmation are required to finish `hosted-entra-session`.
2. **Honest unavailable state:** Sociobot's recurring Dock product mapping is
   incomplete. Checkout is disabled and its current unavailable-state claim
   passes. This is not an M1 defect.
3. Physical scanner/camera pilot coverage and customer evidence remain venture
   experiments in the plan, not shipped compatibility certifications.

Final totals: **1 finding; 1 untested claim; verdict FAIL.**
