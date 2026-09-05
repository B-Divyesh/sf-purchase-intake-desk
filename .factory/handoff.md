# Factory handoff — repair 4

Date: 2026-09-05
Controller stage: `building-m1`
Milestone: M1 public demo and receiving workflow; no later milestone was added.

## Status

**Deployed and verified, with one named external CIAM dependency still unverified.** The M1 job works on desktop and phone. The Dock sign-in code is configured and its hosted redirect begins correctly, but the full provider sign-in/session/sign-out claim cannot be completed without a dedicated CIAM test account and callback-registration confirmation.

- Implementation commit: `64df103df5f62e428da00cd4e25d6e3f47e6cae1`
- Documentation commit: `ae9873e66ea093f3aa2bf58355fe6e352dbfcb53`
- Live URL: <https://purchase-intake-desk.sociobot.in>
- Live implementation image: `sociobotregistry.azurecr.io/sf-purchase-intake-desk@sha256:3fb074577030ac81a821290037a004738e4c02a2935a6d53bb2c36f84382c293`
- Live revision: `sf-purchase-intake-desk--0000012`

## What changed

- `/ready` now verifies the active `/data/intake-desk-rollback.sqlite3` file, a real SQLite read/write lock, and a writable `/data/objects` evidence directory. It no longer looks for the obsolete WAL filename.
- Added an isolated-volume integration test that starts an empty durable directory, receives readiness, detects a missing evidence directory, restarts on the same directory, and reads a persisted marker.
- Test fixtures now use the same active rollback-journal startup path as the release service.
- README storage instructions now name the active database and `DELETE` rollback journal.
- Added the outcome claim proving demo reset leaves a separately imported workspace unchanged.
- Added an explicit live hosted-CIAM claim. It runs only with an isolated test account and live origin, and reports `skipped` rather than a false pass when those external inputs are absent.

## Verification

From a clean dependency install:

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

- `npm ci`: 86 packages, 0 vulnerabilities.
- `npm run check`: 0 errors and 0 warnings.
- `npm test`: 17 web checks and 14 Rust checks passed.
- `npm run test:e2e`: 44 passed across desktop and 390×844 phone; the two hosted-CIAM project executions skipped because this worker has no provider test account.
- `npm run build`: passed; `dist/` and release binary produced.
- Rust formatter and clippy with warnings denied: passed.
- The fresh isolated-volume regression passed. A separately launched release binary created the rollback database and objects directory, returned 200 for health/readiness, shut down cleanly, restarted from that same directory, and returned readiness again.
- The 23 claim entries are all declared. The 22 claims without an external identity pass; `hosted-entra-session` is an honest skip pending the named provider inputs below.

## Live evidence

- `GET /health` and `GET /ready` each returned 200 and build `64df103df5f62e428da00cd4e25d6e3f47e6cae1` after deployment.
- Deployment inspection confirms one replica, `/data` mounted from this product's `sf-purchase-intake-desk-data` Azure Files share, and the immutable digest above. The deployed revision retained the durable mount.
- `/opt/fleet/lib/verify-url.sh` passed in 679 ms: title, `lang=en`, one H1, main landmark, image alt text, button labels, and console checks all passed. Desktop and 390 px screenshots are under `/work/.evidence/purchase-intake-desk-repair4/`.
- In both fresh screenshots, before scrolling: the job is “Check deliveries against the purchase order”, the audience is small receiving teams, and the first action is “Try it with sample data”. One click showed populated Northline Bearings PO NB-1047, the persistent sample label, Reset demo, and Start for real.
- The live browser suite passed 44 checks with the two correctly skipped hosted auth checks. Its Axe route scans cover desktop and phone public/demo/legal routes with no violations.
- A 100-request forwarded-client burst produced 91×401 and 9×429. A separate 160-request parallel burst returned 429 with `Retry-After: 0`; after two seconds the same client was admitted and received the expected 401.
- A live Sign in click opened the Sociobot CIAM authority with authorization code PKCE, the expected client ID, scopes, and this product's callback URI. No identity credentials were entered or logged.

## Earlier findings

| Earlier finding | Current disposition |
| --- | --- |
| Receipt output ignored changed counts; decimal output was inexact; impossible damage finalized | Fixed and covered by existing outcome claims. |
| Real workspace did not retain multiple PO receipts/evidence | Fixed for the current local workspace; retention claim passes. |
| Tenant data could collide; reopened records, audit corrections, evidence, backup, and entitlement behavior were unsafe | Fixed by tenant-scoped persistence and existing server claims. |
| Authenticated API responses could enter shared cache; 401 lacked Bearer challenge | Fixed and covered by cache/HTTP checks. |
| Static cache, 404, metadata, landmark, touch, camera and checkout-copy findings | Fixed or accurately represented as unavailable; browser and route checks pass. |
| Clean durable volume never became ready | Fixed by the active-storage readiness probe and fresh-volume/restart regression. |
| README named the obsolete SQLite file and WAL mode | Fixed in documentation commit `ae9873e`. |
| Demo reset versus workspace boundary had no explicit claim | Fixed by `demo-reset-preserves-workspace`. |

## External dependencies and known limits

1. Hosted CIAM requires an isolated product test account in `ENTRA_E2E_USERNAME` and `ENTRA_E2E_PASSWORD`, plus confirmation that `https://purchase-intake-desk.sociobot.in/auth/callback` is registered on the shared Sociobot Entra SPA. Run the documented `@claim:hosted-entra-session` command when those inputs exist. No shared production credential was used.
2. Sociobot's recurring Dock product mapping remains incomplete. Checkout is deliberately unavailable; no payment, fake license, or direct payment path was added. The planned price remains $149 USD per site each month.
3. Supplier email, accounting/ERP sync, and later operations work are not shipped or claimed in M1.
