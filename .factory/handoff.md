# Factory handoff — repair 3

## Independent verification 4 — 2026-09-05

**Verdict: FAIL.** Implementation `5e3cef391ac4811d9c6439ea781f875adfda9aac`
was reviewed from documentation commit
`79f188842b421e9cafbf12c57c4d32ad20145ea2`. All 21 declared claim commands,
30 unit/integration tests, 42 browser checks, the production build, formatter,
clippy, live phone/desktop flows, axe route scan, rate limiting, and Lighthouse
passed.

Three findings remain:

1. A clean data directory creates `intake-desk-rollback.sqlite3`, but `/ready`
   still checks for `intake-desk.sqlite3`; fresh start and restart both return
   503. Production is masked by the retained old file.
2. The README still documents the old database filename and WAL mode instead
   of the repaired rollback-journal storage.
3. The public hosted Entra sign-in/sign-out path is absent from
   `.factory/claims.json` and remains untested without an operator identity.
   The demo-reset/workspace boundary is also public and needs its own claim
   entry, although it passed a live manual check.

Fresh Lighthouse scored 98 performance, 100 accessibility, 100 best practices,
and 100 SEO. Full evidence and earlier-finding dispositions are in
`.factory/verification-4.md` and `.factory/evidence-verification-4/`.

The external dependencies are unchanged: an operator must provide a CIAM test
identity and confirm the callback registration; Sociobot must complete the
recurring-product mapping before checkout is enabled.

## Status

Repair complete and deployed. The live implementation is commit
`5e3cef391ac4811d9c6439ea781f875adfda9aac` at
<https://purchase-intake-desk.sociobot.in>.

Dock checkout remains intentionally unavailable. The page states the planned
$149/site/month price but offers no checkout link, token storage, or simulated
paid state. Sociobot must complete the separate product mapping before that
path can be enabled.

## What changed

- Replaced global PO, receipt, event, attachment, and entitlement keys with
  tenant-and-site-scoped persistence. The same IDs can now exist in two
  tenants without collision.
- Corrected membership lookup so an existing staff member opens the assigned
  site. Signed-in clients now reload server purchase orders, finalized
  receipts, audit events, and retained evidence metadata.
- Replaced fixed audit times with UTC request times. Server corrections append
  immutable events, retrying finalization is idempotent, evidence can be
  downloaded, and SQLite's backup API creates a consistent snapshot.
- Enforced active entitlements on every protected write. Lapsed, revoked, or
  expired sites remain readable and return `402 entitlement_required` for
  writes. Export remains local and available.
- Removed the broken production checkout call. License attachment returns a
  clear `503 checkout_unavailable` until the external mapping exists.
- Prevented the service worker from reading or writing `/api`, health, ready,
  or authorized-request cache entries. Online navigation is network-first, so
  unknown live URLs remain real HTTP 404 responses.
- Added CIAM session restore and sign-out, one-hour discovery/JWKS caching,
  `WWW-Authenticate: Bearer` on 401 responses, checked server errors, and
  read-only messaging.
- Rejected zero quantities, removed duplicate metadata, fixed the nested
  landmark, covered the empty workspace, and made the 404 page use the shared
  site structure and plain wording.
- Migrated the mounted database to a rollback-journal file opened with
  SQLite's single-process `unix-excl` VFS. The migration seeds from
  `backup-latest.sqlite3` after an integrity check. The old WAL database and
  sidecars remain untouched. This is compatible with the one-replica Azure
  Files deployment.

## Verification

From a clean checkout of the implementation SHA:

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
- `npm test`: 17 web tests and 13 Rust unit/integration checks passed.
- Every command in `.factory/claims.json`: 21/21 passed from the clean clone.
- `npm run test:e2e`: 42/42 passed on desktop and 390 px phone, including axe,
  keyboard focus, touch targets, offline reload, service-worker update, demo
  isolation, invalid values, reset, exports, and designed 404 states.
- Formatter and clippy: passed with warnings denied.
- Production build: initial app JavaScript 35.00 KB gzip, CSS 5.04 KB gzip;
  the 60.86 KB gzip CIAM chunk loads only after sign-in starts.

Outcome regressions cover two tenants using the same PO/receipt IDs, a shared
member reopening records from a fresh process, current audit times and
correction history after reopen, entitlement read-only behavior, evidence
retrieval, snapshot integrity, rate limiting, and authenticated cache bypass.

## Live evidence

- Image: `sociobotregistry.azurecr.io/sf-purchase-intake-desk:5e3cef391ac4`
- Digest: `sha256:1df8a0503303ee30153435b0905320d3e647ca21a3792677032cff3a9a6b15ef`
- Revision: `sf-purchase-intake-desk--0000010`, healthy, 100% traffic.
- Scale: minimum 1, maximum 1. Durable
  `sf-purchase-intake-desk-data` is mounted at `/data`.
- `/health` and `/ready` returned the full implementation SHA before and after
  an explicit revision restart. The replacement replica was ready with zero
  restarts.
- A 100-request same-IP burst produced 90 `401` responses and 10 `429`
  responses. Every 429 included `Retry-After`.
- Public routes, legal pages, robots, sitemap, and `/sw.js` returned 200.
  `/api/v1/me` returned 401 with `WWW-Authenticate` and `no-store`. The tested
  unknown URL returned the designed page with HTTP 404.
- Fresh desktop and phone contexts stated the job, audience, and first action
  before scrolling. Each opened populated sample NB-1047 in one click, showed
  the persistent demo label, changed a count, reset it to 46, used only
  `intake-desk:demo:v1`, and sent no API request.
- Live verification found no unexpected console error and valid title, lang,
  one H1, main landmark, alt text, and labeled buttons.
- Lighthouse mobile: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 2.15 s, CLS 0, total blocking time 32 ms.

Evidence is in `.factory/evidence-repair-3-live/`. The full disposition of the
previous verification findings is in `.factory/repair-3-verification.md`.

## Known limits and operator action

- Confirm/register
  `https://purchase-intake-desk.sociobot.in/auth/callback` on the shared CIAM
  SPA. No QA identity was available, so an interactive hosted sign-in was not
  attempted. JWT validation and tenant behavior use recorded/test identities
  only.
- Complete the separate Sociobot recurring-product mapping before enabling
  checkout or license attachment. No production checkout was accessed.
- The old WAL database is retained on `/data`. The new database was seeded
  from the latest consistent finalized-receipt snapshot. If the old deployment
  held later unfinalized drafts, an operator should archive and inspect the old
  files offline; this repair does not delete or guess at those records.
- No supplier email, accounting sync, or ERP integration is claimed. Those
  remain later product milestones.

## Next step

After CIAM callback and billing mapping are confirmed, run one real test-tenant
sign-in and staging checkout, then repeat the two-tenant browser workflow. Do
not enable production checkout before that external mapping is verified.
