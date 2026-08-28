# Demo sandbox contract

Status: planned for M1; the scaffold does not implement the demo yet.

## Entry and routing

- Verification entry: `https://purchase-intake-desk.sociobot.in/?demo=1`.
- Local entry after M1: `http://localhost:5173/?demo=1`.
- `/?demo=1` enters sample data immediately and canonicalizes to
  `/demo?demo=1`. `/demo` also enters demo mode.
- A persistent banner says: “Demo — sample data, nothing is saved.” It offers
  “Reset demo” and “Start for real.”

## Sample data

The deterministic sample is West Yard receiving PO NB-1047 from Northline
Bearings against packing list NL-8821:

| Code | Expected | Demo delivery |
| --- | ---: | ---: |
| BRG-6204 | 120 each | 120 each |
| BLT-A42 | 4 cases × 12 = 48 each | 46 each; 2 short |
| SEAL-28 | 24 each | 24 each; 1 damaged |

The flow includes local sample evidence, a partial-delivery discrepancy,
finalization at 2026-08-28 09:40 site time, a correction event fixture, and a
receipt CSV download. It must look in use on the first screen; it never starts
as an empty tutorial.

## Isolation and reset

- IndexedDB database: `intake-desk:demo:v1`.
- Cache namespace: `intake-desk:demo-shell:<build-id>`.
- Production storage later uses
  `intake-desk:tenant:<tenant_id>:site:<site_id>:v1`; demo code cannot open it.
- Demo network traffic is same-origin static content only. It never calls
  auth, billing, production APIs, email, telemetry, or AI.
- Reset closes/deletes the demo database, recreates the exact seed, clears only
  demo cache/data, and returns to the demo inbox. It cannot clear production
  stores.
- “Start for real” leaves demo, discards demo changes, and goes to the public
  start/sign-in route. M1 explains that accounts are not built yet; M2 enables
  sign-in. Demo records are never silently copied into production.

## Offline verification

The first online visit installs the versioned service worker and caches the
shell plus sample. A clean Playwright context opens `/?demo=1`, waits for the
worker, goes offline, reloads, completes/finalizes a receipt, opens it again,
and downloads CSV without a network request. Reconnect must not send demo data.

## Claim verification

Every M1 entry in `.factory/claims.json` starts from a fresh browser context
using only this sample. Tests may change it during a case, then reset. No test
depends on a user account, secret, external service, or another test's state.
