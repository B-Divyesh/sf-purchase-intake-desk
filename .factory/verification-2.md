# Independent product verification 2

**Verdict: FAIL — do not release candidate `0a7243f638a836490549a90bdf80295e189dc8e1`.**

Verified 2026-08-28 from a clean checkout at that exact commit against
`https://purchase-intake-desk.sociobot.in` (demo entry
`https://purchase-intake-desk.sociobot.in/?demo=1`). No product code was
changed. The live `/health` returned the complete candidate SHA, and the
SHA-256 of live `assets/index-D4nZbTOg.js` exactly matched the artifact built
with the candidate SHA:

```text
7d58c47bfdb234c978d098d8ef172eedeefb65dc11a52540dd25a8d10d850ca5
```

The prior report is `verification.md`; this report supersedes its PASS repair
handoff for release acceptance.

## Required gates

### Claim tests — PASS

`.factory/claims.json` exists and contains 13 claims. After `npm ci`, I ran
every declared command separately from the demo entry point, in both configured
desktop and 390 px Playwright projects. Every command passed (26 test
executions):

| Claim | Result |
| --- | --- |
| `demo-entry-reset` | PASS |
| `expected-vs-received` | PASS |
| `exact-unit-conversion` | PASS |
| `partial-discrepancy` | PASS |
| `immutable-correction` | PASS |
| `receipt-csv-export` | PASS |
| `offline-reload` | PASS |
| `demo-isolation` | PASS |
| `scanner-manual-fallback` | PASS |
| `current-count-finalization` | PASS |
| `exact-decimal-export` | PASS |
| `damage-count-validation` | PASS |
| `real-po-workspace` | PASS |

The complete browser suite also passed; `test-results/.last-run.json` reports
`"status": "passed"` and no failed tests.

### First read — PASS

A cold desktop visit plainly answers all three questions on the first screen:

- **What:** “Check deliveries against the purchase order.”
- **For whom:** “For small receiving teams that need a clear record before the supplier van leaves.”
- **What to click:** “Try it with sample data”, with “Opens one ready PO. No account.” beside it.

The one-click demo opens a populated Northline Bearings PO and shows the
persistent sample-data banner. This worked at 390 px as well. Evidence:
`evidence-verification-2/live-first-read-desktop.png`.

## Release-blocking defects

### Critical — a second real PO destroys the prior real receipt and evidence

The real workspace is not a PO inbox or a retained receipt store. It has one
IndexedDB record: database `intake-desk:workspace:v1`, object key
`active-purchase-order`. Importing a second PO replaces that record.

Fresh live reproduction:

1. Imported and finalized `PO-220`.
2. Imported `PO-221` in the same browser workspace.
3. The stored record became `PO-221`; the PO-220 route rendered PO-221’s count
   screen and PO-220 was no longer available.

This loses the prior receipt/evidence as soon as the next delivery is imported.
It violates the brief’s supplier PO inbox and document-retention constraints,
and makes the claimed audit trail unsuitable for a receiving team. Source
confirms the single fixed key in `src/lib/storage/workspaceRepository.ts`.

### Critical — this is not the required shared, durable web-with-backend product

The deployed Rust service exposes `/health` and static-file fallback only. It
has no receipt, import, attachment, audit, tenant, or account endpoint; all
real data and evidence remain only in one browser’s IndexedDB. There is no
Sociobot Entra CIAM sign-in, site/member access, server-side tenant isolation,
durable object storage, backup/export of a workspace, or subscription billing.

The product itself discloses a “current single-device workspace”, but that is an
M1 pilot boundary, not the researched contract for 10–100-person teams paying
$149/site/month. It cannot meet the brief’s real receiving job, persistence,
or team workflow end to end. In particular, a browser-controlled event list is
not an immutable receipt audit trail.

### High — phone/QR receiving is not implemented

The brief explicitly requires QR/phone receiving and offline-capable scanning.
The receive screen supports typed and keyboard-wedge codes, but its only camera
control is “Check camera fallback”. `tryCamera` merely probes camera permission
and then says “This demo uses typed or keyboard scanner codes”; it contains no
barcode/QR decoder or camera scan path. This is a missing core acceptance
feature, not a browser-compatibility fallback.

### High — the advertised $149 Dock plan is unimplemented and unclaimed

The landing page promises “Dock plan: $149 per site each month”, while the
repository has no subscription checkout, Sociobot billing integration, license
or entitlement check, or paid-plan claim test. `claims.json` has no entry for
this price/plan promise. Under the supplied claims contract, an unlisted
testable landing/README claim is release-blocking; under the researched brief,
the declared subscription is also not deliverable.

## Verified behaviour that does work

- With all demo lines changed to matched, the finalization bar, confirmation,
  receipt, and audit text correctly record a complete delivery with no
  discrepancy.
- `46.1` against `48` exports `-1.9` exactly in receipt CSV.
- Damage greater than received blocks finalization, announces the error, and
  focuses the error summary.
- A valid customer CSV imports with exact unit conversion; a valid PNG evidence
  attachment is kept with that current workspace receipt. An invalid CSV gives
  a focused, plain error and recovery action.
- Demo and real workspace browser flows made same-origin requests only. The
  cold landing request log contained only the product document, local assets,
  and self-hosted fonts; no analytics, third-party scripts, billing, AI, or
  external data service was observed.
- The live service worker controlled the demo, used cache
  `intake-desk:demo-shell:0a7243f638a83649`, and retained an edited count
  through an offline reload.
- Live rate limiting is active on non-health requests: 120 concurrent
  `/robots.txt` requests returned 50×200 and 70×429. The observed allowance was
  `X-RateLimit-Limit: 40`; each 429 included `Retry-After: 0`. `/health` is
  exempt as documented.
- Live responses set HSTS, CSP with `connect-src 'self'`, `X-Content-Type-
  Options: nosniff`, and a strict referrer policy. HTML and `sw.js` are
  no-store/no-cache; hashed assets and self-hosted fonts are immutable for one
  year; an unknown route returns HTTP 404.
- Desktop and 390 px routes had one `<main>` and one `<h1>`, `lang=en`, no
  overflow, no normal-route console/page errors, and no axe serious/critical
  findings. Keyboard testing confirmed a visible 3 px focus ring, working skip
  link, dialog focus management, and focus return. Axe did report the landing
  page’s `landmark-complementary-is-top-level` issue at **moderate** impact;
  it is non-blocking relative to the requested serious/critical gate.

## Local verification

All completed successfully from the candidate tree:

```text
npm ci
npm run check
npm test                         # 9 files, 16 web tests; 5 Rust tests
npm run test:e2e                 # passed, no failed tests
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
BUILD_SHA=0a7243f... npm run build
```

The SHA-stamped production web build is 31.70 KB gzip JavaScript and 4.93 KB
gzip CSS. The three self-hosted font files total 111.3 KB, within the stated
budgets. The release API also started with an otherwise empty environment and
only `PORT=18080`; `/health` returned the candidate SHA, unknown routes returned
404, and hashed asset caching was immutable.

The Docker CLI is not installed in this verification container, so an image
build/run could not be performed. Static inspection found a compliant
multi-stage Dockerfile with `rust:1-slim`, build-identity args, a non-root
distroless runtime, and no `.git` copy.

## Evidence

- `evidence-verification-2/live-first-read-desktop.png`
- `evidence-verification-2/live-matched-receipt.png`
- `evidence-verification-2/live-demo-mobile-reduced.png`
- `evidence-verification-2/live-keyboard-focus-mobile.png`
- `evidence-verification-2/live-headers.txt`
- `evidence-verification-2/live-index-D4nZbTOg.js`

## Required next work

Do not release this as the paid/team product. Implement the planned M2/M3
server-owned tenant data model, Entra CIAM, multi-PO inbox, durable attachment
storage/retention, receipts that cannot be overwritten by later imports,
Sociobot subscription entitlement, and a real camera QR/barcode scan path.
Add claim tests for each restored promise and for importing multiple POs while
retaining and exporting each earlier receipt.
