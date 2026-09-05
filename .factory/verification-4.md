# Verify supplier delivery intake — independent verification 4

**Verdict: FAIL — 3 findings, including 2 high-severity findings.**

Verified on 2026-09-05 against:

- Implementation reviewed: `5e3cef391ac4811d9c6439ea781f875adfda9aac`
- Documentation checkout: `79f188842b421e9cafbf12c57c4d32ad20145ea2`
- Live URL: <https://purchase-intake-desk.sociobot.in>
- Controller stage: `building-m1`

The M1 receiving job passes on phone and desktop. The repaired tenant data,
audit, evidence, entitlement, and cache boundaries also pass their declared
tests. The candidate is not accepted because a clean data volume can never
become ready, the README describes an obsolete database path and journal mode,
and the public hosted sign-in promise is not in the claim inventory or tested
with a real identity.

No product code was changed during this verification.

## Findings

### High — a clean data volume never becomes ready

Repair 3 moved the live database to `intake-desk-rollback.sqlite3` with SQLite
`DELETE` journal mode. The readiness handler still checks for the removed
`intake-desk.sqlite3` path.

- `api/src/lib.rs:311-318` opens `intake-desk-rollback.sqlite3`.
- `api/src/lib.rs:590-600` returns ready only when
  `intake-desk.sqlite3` exists.
- In a new temporary data directory, the release binary started, `/health`
  returned 200, and `intake-desk-rollback.sqlite3` was created.
- `/ready` returned `503 {"code":"not_ready",...}` before and after a clean
  process restart.

Production `/ready` returns 200 only because the durable volume retains the old
database file. That masks the defect. A new site or recovered empty volume
would remain unready even though its active database is usable.

### Medium — the README gives the wrong storage path and journal mode

`README.md:56-60` says the server creates `/data/intake-desk.sqlite3` with WAL
storage. The current implementation creates
`/data/intake-desk-rollback.sqlite3` and explicitly sets `journal_mode=DELETE`
at `api/src/lib.rs:303-305`. The repair handoff describes the new behavior, but
the primary run documentation does not.

### High — the public sign-in claim is missing and remains untested

The site exposes **Sign in**, the README says an entitled Dock site uses
Sociobot Entra sign-in, and the privacy page makes account-key and signed-in
request claims. `.factory/claims.json` has no sign-in/sign-out claim. This was
also an explicit M2 claim in the venture plan.

A fresh browser proved that the button starts PKCE at the specified
`sociobotcustomers.ciamlogin.com` tenant with the expected client ID, callback,
scopes, state, and S256 challenge. Unit fixtures cover JWT and tenant behavior.
The hosted login, callback, restored session, and sign-out could not be
completed without the operator test identity. This leaves **1 untested public
claim** and fails the claims contract. The terms statement that demo reset does
not remove the separate workspace was verified live, but it also needs an
explicit claim entry rather than only incidental coverage.

## First screen and live job

Fresh 1440×900 and 390×844 profiles showed these before scrolling:

- Job: **“Check deliveries against the purchase order.”**
- Audience: **“For small receiving teams that need a clear record before the supplier van leaves.”**
- First action: **“Try it with sample data.”**

One click opened Northline Bearings PO `NB-1047` with three realistic lines.
The persistent label said **“Demo — sample data, nothing is saved”** and kept
**Reset demo** and **Start for real** visible.

On desktop, I finalized the seeded 2-each shortage and 1-each damage, opened
the combined discrepancy, downloaded a four-row CSV, and appended a correction.
The original event hash did not change. On phone, impossible damage was blocked
with focused guidance, an unknown item code recovered to `BRG-6204`, camera
denial gave the typed/scanner fallback, and offline reload retained a changed
count of 45. Reset restored the belt count to 46.

Both demo profiles made zero `/api` requests and zero third-party requests.
No workspace database appeared. A separate live workspace check imported
`PO-401`, rejected a zero quantity, converted 2.5 cases × 20 to 50 each,
rejected a false PNG signature, and remained present after demo reset.

Evidence:

- `evidence-verification-4/first-screen-desktop.png`
- `evidence-verification-4/first-screen-phone.png`
- `evidence-verification-4/demo-desktop.png`
- `evidence-verification-4/demo-phone.png`

## Declared claims

All 21 commands in `.factory/claims.json` were run separately after `npm ci`.
All declared claims passed. Each browser claim passed in desktop Chromium and
the 390×844 phone project.

| Claim | Result |
| --- | --- |
| `demo-entry-reset` | PASS, 2/2 projects |
| `expected-vs-received` | PASS, 2/2 |
| `exact-unit-conversion` | PASS, 2/2 |
| `partial-discrepancy` | PASS, 2/2 |
| `immutable-correction` | PASS, 2/2 |
| `receipt-csv-export` | PASS, 2/2 |
| `offline-reload` | PASS, 2/2 |
| `demo-isolation` | PASS, 2/2 |
| `scanner-manual-fallback` | PASS, 2/2 |
| `current-count-finalization` | PASS, 2/2 |
| `exact-decimal-export` | PASS, 2/2 |
| `damage-count-validation` | PASS, 2/2 |
| `real-po-workspace` | PASS, 2/2 |
| `multi-po-retention` | PASS, 2/2 |
| `checkout-unavailable` | PASS, 2/2 |
| `phone-qr-scanning` | PASS, 2/2 |
| `server-tenant-isolation` | PASS |
| `server-record-reload` | PASS |
| `server-audit-retention` | PASS |
| `entitlement-read-only` | PASS |
| `authenticated-cache-bypass` | PASS, 2/2 |

Claim result: **21/21 declared claims passed; 1 public claim remains untested
because it is not declared.**

## Earlier finding disposition

| Earlier finding | Current evidence | Disposition |
| --- | --- | --- |
| Final records ignored entered counts | Matched-count claim passed in both browser projects; live seeded output used current values. | Fixed |
| Decimal CSV was inexact | `46.1 - 48` exported as `-1.9` in both projects. | Fixed |
| Damage could exceed received | Live phone and both projects blocked it and focused the error. | Fixed |
| No real PO or evidence path | Live import, unit conversion, evidence validation, finalization, and retention worked locally on the device. | Fixed for current local workspace |
| A second PO destroyed the first receipt | `multi-po-retention` passed twice. | Fixed |
| Touch targets were too small | The 44 px browser checks passed at desktop and phone sizes. | Fixed |
| Static caching was missing | Live hashed JS is immutable for one year; HTML is no-store. | Fixed |
| Unknown routes returned 200 | `/404` and a fresh unknown URL returned a designed page with HTTP 404. | Fixed |
| Docker pinned a Rust minor | Dockerfile uses `rust:1-slim`. | Fixed |
| Service-worker cache was not build-versioned | Both projects passed the update/cache-name check. | Fixed |
| Phone camera scan was missing | Recorded camera fixture passed twice; live denial fallback passed. | Fixed |
| Tenant IDs could overwrite each other | Exact two-tenant claim passed. | Fixed |
| Assigned users could not reload server records | Exact reopen/member claim passed. | Fixed |
| Audit time, corrections, evidence retrieval, and backup were unsafe | Exact audit-retention claim passed. | Fixed |
| Entitlements were not enforced | Exact read-only entitlement claim passed. | Fixed |
| Authenticated responses entered the shared cache | Cache-poisoning claim passed twice. | Fixed |
| 401 lacked Bearer challenge | Live 401 includes `WWW-Authenticate: Bearer`. | Fixed |
| Zero quantities imported | Live zero-quantity import was rejected with a focused error. | Fixed |
| Route metadata was duplicated | Every checked route had one description and one canonical link. | Fixed |
| Landing preview used a nested complementary landmark | Axe found zero violations on all checked routes. | Fixed |
| Checkout was broken or priced incorrectly | The current $149 price is labeled planned; checkout is disabled and absent as a link. | Fixed for the declared unavailable state |
| Durable restart behavior | Server reopen claim passes; the prior live restart retained data. | Data persistence fixed; fresh-volume readiness finding remains |

## Accessibility, routes, privacy, and performance

- `/`, `/demo`, `/start`, `/app`, `/privacy`, `/terms`, `/404`, and a fresh
  unknown route each had one H1, one main landmark, one description, one
  canonical link, no horizontal overflow, and zero axe violations.
- The first Tab reached the visible skip link with a designed outline. Dialog
  errors received focus. The full suite passed keyboard focus and 44 px target
  checks on both viewports.
- The phone reduced-motion profile had zero running animations. Dark mode and
  offline reload passed.
- All discovered internal links returned 200, except the deliberate `/404`
  link, which correctly returned 404. Legal mail links were present.
- The URL verifier passed in 610 ms with `lang=en`, one H1, a main landmark,
  no missing image alt text, no unlabeled buttons, and no console error.
- Fresh Lighthouse: performance 98, accessibility 100, best practices 100,
  SEO 100; LCP 2.1 s, CLS 0, TBT 20 ms.
- Initial JS is 104.59 KB raw / 34.99 KB gzip; CSS is 22.08 KB raw / 5.04 KB
  gzip; self-hosted fonts total 111.33 KB. The CIAM chunk is deferred.

The deliberate 404 produced the browser's expected failed-resource console
line for its navigation response. It rendered the designed page and is not a
defect.

## Backend and live identity

- Live `/health` and `/ready` returned 200 with build
  `79f188842b421e9cafbf12c57c4d32ad20145ea2`.
- The only changes between implementation `5e3cef3` and documentation
  `79f1888` are reports and evidence. A build stamped with `79f1888` produced
  `assets/index-CwcPfcD-.js`; its SHA-256 exactly matched production:
  `3bbb0352efcfd5f9795727009a215811368c21d9b2b09e221c9bc37071ed3c54`.
- A 160-request burst from one forwarded client produced 44×401 and 116×429.
  Every 429 included `Retry-After: 0`. After two seconds, the next request was
  admitted and returned the expected 401. Eighty concurrent health checks all
  returned 200.
- Live API 401 responses include the Bearer challenge and `no-store`. Security
  headers include CSP, HSTS, `nosniff`, strict referrer policy, and a camera-only
  permissions policy.
- Tenant isolation, reopened records, current audit times, append-only
  corrections, evidence retrieval, consistent backup, and entitlement
  read-only behavior passed exact temporary-SQLite tests.
- A temporary release process shut down cleanly on SIGTERM. Its restart retained
  the rollback database, but readiness remained 503 as described above.

## Local quality gates

| Command | Result |
| --- | --- |
| `npm ci` | PASS; 86 packages, 0 vulnerabilities |
| `npm run check` | PASS; 0 errors, 0 warnings |
| `npm test` | PASS; 17 web tests and 13 Rust tests |
| Every declared claim command separately | PASS; 21/21 |
| `npm run test:e2e` | PASS; 42/42 |
| `npm run build` | PASS; `dist/` and release binary produced |
| `cargo fmt --manifest-path api/Cargo.toml -- --check` | PASS |
| `cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings` | PASS |
| `/opt/fleet/lib/verify-url.sh` | PASS |

## Current milestone and external dependencies

Controller stage `building-m1`: the M1 public demo and local receiving job pass.
The deployment also exposes repaired M2 server behavior, so its current public
promises were checked. M3 supplier email, accounting integration, and later
operations features were not required or presented as shipped.

External dependencies are separate from the code findings:

- Hosted CIAM needs an operator test identity and confirmed callback
  registration. The redirect configuration is correct, but the complete user
  flow remains untested and must be added to the claim inventory.
- Sociobot recurring-product mapping is incomplete. Checkout is deliberately
  unavailable, clearly labeled, and its declared unavailable-state claim
  passes. No payment path was attempted.

## Required repair

Make `/ready` check the active rollback database or, better, verify the open
connection and object directory. Add a clean-volume readiness regression test.
Update the README path and journal-mode wording. Add and run an explicit hosted
CIAM sign-in/session/sign-out claim with the operator identity, plus an explicit
demo-reset-versus-workspace isolation claim.
