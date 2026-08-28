# Factory handoff — release-blocking QA repair

**Status: PASS — repair deployed and verified.**

Repair work order `purchase-intake-desk-repair-1` addressed every finding in
the independent report at `c8bbb897776d577f7c7587a008eed075db0864e1` for
candidate `3d5044ccf64cb9b643b054a9a78c99018acfbd11`.

Deployed product source: `c50dc74745be2784c8035e7d460ea75adc53aeb6`.
Live URL: <https://purchase-intake-desk.sociobot.in>.

## What changed

- One receipt-summary domain function now derives the finalization bar,
  confirmation, receipt totals/state, discrepancy lines, and audit text from
  the receiver's current values. A fully matched edit records zero shortages,
  zero damage, `Complete delivery`, no discrepancy link, and an accurate
  hash-linked event.
- CSV differences now use decimal-string arithmetic. `46.1 - 48` exports as
  `-1.9`, never a binary floating-point residue. Stepper changes use the same
  exact arithmetic.
- Finalization rejects damaged quantity greater than received quantity and
  announces/focuses the error.
- `/start` accepts a real supplier PO CSV, validates the complete file,
  performs exact case conversion, and writes only to
  `intake-desk:workspace:v1`. The real flow supports scanner/manual counts,
  photo/PDF evidence with size/type/signature checks and SHA-256, discrepancy
  records, immutable receipt history, and receipt CSV export. The demo remains
  in the isolated `intake-desk:demo:v1` namespace.
- The claim inventory now has 13 entries and exactly one matching browser test
  per claim. The unsupported README claims called out by the verifier were
  removed; current real-workspace and integrity claims were added.
- All visible links and controls now meet the 44×44 px target baseline. The
  browser suite measures every visible interactive element on seven routes at
  desktop and 390×844.
- The container returns an actual HTTP 404 for unknown routes while preserving
  successful SPA deep links. HTML and `sw.js` use `no-cache`; hashed assets and
  fonts use `max-age=31536000, immutable`.
- The Docker builder now uses `rust:1-slim`. Vite stamps the service-worker
  cache with the build SHA, and activation removes prior shell caches.

## Exact regression coverage

- `@claim:current-count-finalization`: repeats the verifier's 48 belts + good
  seals case and checks the bar, dialog, receipt, absent discrepancy action,
  and audit event.
- `@claim:exact-decimal-export`: enters `46.1` against `48`, downloads CSV,
  asserts `-1.9`, and rejects `-1.8999999999999986`.
- `@claim:damage-count-validation`: enters received `1`, damaged `2`, asserts
  no confirmation dialog, and checks the focused live error.
- `@claim:real-po-workspace`: imports a real CSV, proves exact `2.5 × 20 = 50`,
  attaches evidence, finalizes, checks same-origin traffic, and verifies the
  workspace never opens demo storage.
- Rust integration tests assert successful SPA deep links, real unknown-route
  404 status, immutable asset caching, health identity, and forwarded-IP rate
  limiting with `Retry-After`.
- Accessibility tests cover eight public/demo routes with axe, route/dialog
  focus, skip-link keyboard operation, 44 px targets, and service-worker update
  cache versioning on desktop and 390×844.

## Clean local evidence

Run on 2026-08-28 from the committed tree:

```text
npm ci                                              PASS (84 packages, 0 vulnerabilities)
npm run check                                       PASS (0 errors, 0 warnings)
npm test                                            PASS (16 Vitest + 5 Rust integration)
npm run test:e2e                                    PASS (34/34; desktop + 390×844)
cargo fmt --manifest-path api/Cargo.toml -- --check PASS
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings PASS
BUILD_SHA=c50dc74 npm run build                     PASS (dist/ + release API)
```

Each of the 13 commands in `.factory/claims.json` was also run separately
from a fresh browser context; every command passed in both projects (26/26).

Built web payload: 93.92 KB JS / 31.70 KB gzip, 21.61 KB CSS / 4.93 KB
gzip, and 111.3 KB self-hosted fonts. Local Lighthouse evidence is
[evidence-lighthouse-repair.json](./evidence-lighthouse-repair.json):
performance 99, accessibility 100, best practices 100, SEO 100, LCP 1.96 s,
CLS 0, TBT 26 ms.

The release binary was also started from a clean environment with only
`PORT=18080`. `/health`, SPA deep links, caching, 404 behavior, and graceful
SIGINT shutdown passed.

## Deployment and live evidence

The factory container deployment built and released:

```text
sociobotregistry.azurecr.io/sf-purchase-intake-desk:c50dc74745be
Container App: sf-purchase-intake-desk
```

- `/health` reports the full source SHA
  `c50dc74745be2784c8035e7d460ea75adc53aeb6`.
- A rebuild with that full SHA produced `assets/index-DLjdggVy.js`; local and
  live SHA-256 both equal
  `2e0643ebb3d5b666d389fbf09315bf277018016085050fbe845b75678733e73d`.
- `/opt/fleet/lib/verify-url.sh` passed HTTPS, title, `lang=en`, one `<h1>`,
  `<main>`, alt text, and console checks. Desktop/mobile screenshots and JSON
  are in `.factory/evidence-repair-live/`.
- Live browser replay passed all-matched finalization, exact decimal CSV,
  impossible-damage rejection, real PO import, same-origin privacy, and an
  offline reload preserving `43.5`. It found 0 console errors, 0 serious or
  critical axe findings, and 0 undersized controls at 390 px.
- A 500-request live burst returned 98×200 and 402×429. Every 429 included
  `Retry-After`. `/health` remains exempt.
- Live response checks found `no-cache, no-store, must-revalidate` on HTML,
  unknown routes, and `sw.js`; hashed JS used
  `public, max-age=31536000, immutable`; `/not-a-real-route` returned 404.
- Live service-worker cache identity is
  `intake-desk:demo-shell:c50dc74745be2784`.
- Live Lighthouse evidence is
  [evidence-lighthouse-live-repair.json](./evidence-lighthouse-live-repair.json):
  performance 98, accessibility 100, best practices 100, SEO 100, LCP 2.14 s,
  CLS 0, TBT 32 ms.

## Privacy, identity, and remaining boundaries

No analytics, advertising, third-party font/script, AI, billing, or external
data request was observed. The demo and real workspace are separate local
stores. CIAM/live identity is not applicable to this repaired single-device
pilot because it has no account or protected route.

The shipped product now performs the real receiving job on customer CSV and
evidence, but it is deliberately single-device. Cross-device accounts,
server-side tenant storage, staff roles, subscription checkout, and supplier
email remain the planned M2/M3 expansion. The UI and legal copy state this
boundary. No package/consumer check applies to this `web-with-backend`
artifact.
