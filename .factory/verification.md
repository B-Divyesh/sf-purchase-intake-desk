# Independent product verification

**Verdict: FAIL — do not release candidate `3d5044ccf64cb9b643b054a9a78c99018acfbd11`.**

Verified on 2026-08-28 against:

- Candidate: `3d5044ccf64cb9b643b054a9a78c99018acfbd11`
- Live URL: <https://purchase-intake-desk.sociobot.in>
- Demo entry: <https://purchase-intake-desk.sociobot.in/?demo=1>
- Clean checkout: `/work/repo`, initially clean and at the candidate commit

No product code was changed. This report and QA evidence are the only repository changes.

## Acceptance summary

The mandatory first-read and declared claim-test gates pass. The release still
fails because the finalized receipt, discrepancy, and audit record do not
reflect the receiver's entered counts. A fully matched delivery is recorded as
the seed's shortage and damage. Decimal CSV output also loses accuracy, and
invalid damaged counts can be finalized. These defects break the central job:
creating a trustworthy supplier-delivery record.

The candidate is also explicitly an M1 sample sandbox. It has no path for a
customer to load a real PO or attach real photo/document evidence, and no
production persistence. That is an honest milestone boundary in the product
copy, but it does not satisfy the repository definition of done or the
researched smallest-useful-product contract for a real receiving desk.

## Mandatory first-read test

**PASS.** A cold 1440×900 production visit answers all three required questions
on the first screen:

- What: **“Check deliveries against the purchase order.”**
- For whom: **“For small receiving teams that need a clear record before the supplier van leaves.”**
- First click: **“Try it with sample data”**, beside **“Opens one ready PO. No account.”**

One click opens `/demo` with populated PO `NB-1047`, supplier Northline
Bearings, packing list `NL-8821`, and the persistent banner **“Demo — sample
data, nothing is saved”** with **Reset demo** and **Start for real**. This also
worked at 390×844. Evidence:
[desktop first read](evidence-first-read-desktop.png) and
[390 px demo](evidence-demo-mobile.png).

## Release-blocking findings

### Critical — finalized records ignore entered quantities

On the live receive screen, I changed `BLT-A42` from 46 to 48 and changed
`SEAL-28` from Damaged to Good. All three line statuses visibly became
**Matched**. Despite that:

- the finalization bar still said **“2 each short · 1 seal damaged”**;
- the confirmation still said it would record those two exceptions;
- the receipt still showed Short 2, Damaged 1, and Partial delivery;
- the hash-linked audit event recorded the same false summary;
- the discrepancy route still presented the seed shortage and damage.

This is reproducible on production and is visible in
[the all-matched confirmation](evidence-defect-all-matched-dialog.png).
The fixed strings are also present in `src/App.svelte` in finalization,
receipt-summary, and discrepancy rendering. A receiving record that contradicts
the entered count is not safe to use.

### Critical — candidate cannot perform the researched real job

The only PO is the fixed demo seed. There is no real PO CSV/email intake,
production tenant/store, photo or document attachment, or customer-data path.
The sample contains a pre-drawn evidence fixture. The brief defines the
smallest useful product as a real supplier PO inbox, phone/scanner receiving,
attachment, discrepancy/partial-delivery workflow, and accounting/ERP export.
The repository definition of done explicitly says “not a demo.” The M1 copy
correctly discloses this boundary, but the candidate is not a releasable product
under the supplied acceptance contract.

### High — decimal CSV values are not exact

With `BLT-A42` expected at 48 and received at `46.1`, the downloaded live CSV
contained:

```text
...,48,46.1,-1.8999999999999986,good,0,...
```

The exact difference is `-1.9`. `receiptCsv` converts decimal strings to
JavaScript `Number`, violating the brief's accurate-unit/decimal constraint and
making accounting export unsafe for decimal quantities.

### High — impossible damage quantities can be finalized

Production accepted `received = 1` and `damaged = 2` for `SEAL-28`, then opened
the finalization confirmation. Validation checks only that damaged quantity is
positive, not that it is less than or equal to received quantity. The same
missing cross-field validation permits false evidence records.

### High — claim inventory is incomplete

All nine declared claims have exactly one matching test tag, but the README
makes additional testable claims with no `.factory/claims.json` entry, including:

- “Responsive day/night design, keyboard operation, and public legal pages.”
- “Every non-health request is limited by client IP, using the first valid
  `X-Forwarded-For` address.”
- “The image builds without `.git`, runs as a non-root user, and starts with
  only `PORT` set.”
- “No secret is stored in this repository or sent by the M1 demo.”

The claims acceptance rule says an unlisted landing/README claim fails review.

## Other findings

### Medium — touch targets below 44 px

At 390 px, **Start for real** is 88×24 px, **Back to purchase orders** is
322×24 px, footer links are 21 px high, and the landing privacy link is 21 px
high. Header links also have widths of 27–42 px even though their heights are
44 px. This does not meet the supplied 44×44 touch-target baseline.

### Medium — production does not set caching policy

HTML, `sw.js`, hashed JS/CSS, and WOFF2 responses have no `Cache-Control`
header. Hashed assets therefore do not receive the required long-lived
immutable caching, while HTML/service-worker revalidation is not explicitly
controlled. Lighthouse remains fast on this small build, but the caching
contract is not met.

### Medium — unknown routes return HTTP 200

`/not-a-real-route` renders the designed not-found screen but responds `200`.
`/404` also responds `200`. The axum fallback serves `index.html` with success
for every unknown path, so this is not a real HTTP 404.

### Medium — Dockerfile pins a forbidden Rust minor tag

The Dockerfile uses `FROM rust:1.98-bookworm`. The supplied backend contract
requires `rust:1-slim` or `rust:1-alpine` and explicitly prohibits minor-pinned
Rust images because factory ACR builds must follow current stable.

### Low — service-worker cache is milestone-named, not build-versioned

`public/sw.js` hard-codes `intake-desk:demo-shell:m1`. The live worker installs,
`registration.update()` completes, and offline reload works, but the cache name
does not change with the build as the design/handoff say. Old dynamic assets
can accumulate and there is no observable upgrade transition to a new cache.

## Claims gate

`.factory/claims.json` exists. After the clean-clone prerequisite `npm ci`, I
ran every listed command separately against the shipped demo entry. All passed
in both configured projects:

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-entry-reset` | PASS | 2/2 Playwright projects |
| `expected-vs-received` | PASS | 2/2 |
| `exact-unit-conversion` | PASS | 2/2 |
| `partial-discrepancy` | PASS | 2/2 |
| `immutable-correction` | PASS | 2/2 |
| `receipt-csv-export` | PASS | 2/2 |
| `offline-reload` | PASS | 2/2 |
| `demo-isolation` | PASS | 2/2 |
| `scanner-manual-fallback` | PASS | 2/2 |

The full browser suite subsequently passed **22/22**. The blocker above is a
coverage gap: the tests finalize only the unchanged seed and therefore do not
prove that recorded results are derived from user input.

## Clean-clone gates

| Command | Result |
| --- | --- |
| `npm ci` | PASS; 84 packages, 0 vulnerabilities |
| `npm run check` | PASS; 0 errors, 0 warnings |
| `npm test` | PASS; Vitest 7 files/11 tests, Rust 3 integration tests |
| `npm run test:e2e` | PASS; 22/22 desktop and phone tests |
| `npm run build` | PASS; `dist/` and optimized Rust binary produced |
| `cargo fmt --manifest-path api/Cargo.toml -- --check` | PASS |
| `cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings` | PASS |

The backend also started from a clean environment with only `PORT=18080`,
served `/health`, and shut down gracefully.

## Live deployment identity

**MATCH.** `/health` returned the full SHA
`3d5044ccf64cb9b643b054a9a78c99018acfbd11`. The footer reports
`Build 3d5044ccf64c · M1`. Rebuilding the web bundle with that SHA produced
`assets/index-Bk0RY6kP.js`; its SHA-256 exactly matched production:

```text
e20db692114f1033f8bdd51080531e18598ac571c84ec3b39405975d3111c0f1
```

## End-to-end and recovery checks

- Default seeded receive → finalize → discrepancy → correction → CSV: passes.
- Search no-result state → Clear search: passes.
- Unknown scanner code → plain error → valid code focuses its count: passes.
- Negative count → announced error summary and focus → zero recovers: passes.
- All-matched delivery → final record: **fails** with false seed discrepancy.
- Decimal export: **fails** exactness.
- Damaged greater than received: **fails** validation.
- Offline reload after entering `43.5`: passes and preserves the value.
- Reset restores the seed and remains in the demo namespace: passes.

## Privacy, security, backend, and PWA

- A fresh full live demo flow made same-origin requests only, including
  service-worker install/cache requests.
- Browser storage contained only IndexedDB `intake-desk:demo:v1`; localStorage
  and sessionStorage were empty. Cache Storage contained
  `intake-desk:demo-shell:m1`.
- No analytics, advertising, AI, billing, auth, email, or third-party runtime
  request was observed. There is no sign-in in M1, so CIAM is not applicable.
- No console or page errors occurred in the tested routes/flows.
- Responses included CSP, HSTS, `nosniff`, Referrer-Policy, and
  Permissions-Policy. The CSP matched observed traffic.
- Rate limiting is live on non-health routes: advertised burst **40**, refill
  **20 requests/second**. An 80-request concurrent burst yielded 43×200 and
  37×429 while tokens refilled; every 429 included `Retry-After: 0`.
- `/health` is intentionally exempt; 100 concurrent health requests all
  returned 200 in 468 ms.
- Service worker is active; explicit `registration.update()` succeeded;
  offline navigation/reload and IndexedDB persistence passed.

## Accessibility, responsive behavior, and performance

- Live axe scans found **0 serious/critical** violations across `/`, demo PO,
  receive, finalized receipt, discrepancy, privacy, terms, and not-found views
  in day desktop and dark 390 px contexts.
- Every scanned route had `lang=en`, one `<h1>`, one `<main>`, no missing `alt`,
  no unnamed button, no horizontal overflow, and no console error.
- Keyboard-only smoke passed: first Tab exposes the skip link; its focus ring is
  3 px and visible; Enter focuses `main`; native dialog focus is contained;
  Escape returns focus to the invoking Reset button. Evidence:
  [keyboard focus](evidence-keyboard-focus.png).
- Reduced motion computed to `1ms` animation and `scroll-behavior: auto`.
- Simulated 200% text at 390 px had no horizontal overflow or clipped control.
  Evidence: [200% text](evidence-mobile-text-200.png).
- Touch-target exceptions are listed above.
- Live Lighthouse mobile: Performance **97**, Accessibility **100**, Best
  Practices **100**, SEO **100**; FCP **1.7 s**, LCP **2.2 s**, TBT **140 ms**,
  CLS **0**, transfer **212 KiB**. Raw evidence:
  [Lighthouse JSON](evidence-lighthouse-live.json).
- Candidate-SHA web build: JS 80.90 KB raw / 28.29 KB gzip; CSS 20.94 KB raw /
  4.83 KB gzip; fonts 111,332 bytes. These pass the stated budgets.

## Required next verification

Re-test from a clean clone after receipt/discrepancy/audit derivation uses the
actual line state, decimal export uses exact decimal arithmetic, and cross-field
quantity validation is added. Add tests that finalize an all-matched case, a
different shortage, decimal quantities, and damaged-greater-than-received.
Resolve the scope/claims, touch-target, cache/header, 404, Docker base, and SW
version findings before changing this verdict.
