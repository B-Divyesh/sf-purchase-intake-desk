# Independent product verification 3

**Verdict: FAIL — do not release candidate `ee012af218c7a77894ef11997f60f8c261c29943`.**

Tested on 2026-08-28 against:

- Clean checkout: `ee012af218c7a77894ef11997f60f8c261c29943`
- Production: <https://purchase-intake-desk.sociobot.in>
- Acceptance contract: the supplied work order, researched brief, and attached
  factory skills

The candidate is deployed and the local/demo workflow is polished, accessible,
fast, and well tested. It still fails the researched paid team product contract.
The live checkout is not registered, the signed-in client cannot load server
records, tenant identifiers allow cross-customer overwrites, and the claimed
server audit/entitlement behavior is neither complete nor covered by claims.

## Mandatory gates

### Claims — PASS as written, but incomplete

`.factory/claims.json` exists. After `npm ci`, every listed `test` command was
run separately from the clean checkout through the demo entry point. Every
claim passed in both desktop Chromium and the 390×844 phone project: 32 claim
executions total.

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
| `multi-po-retention` | PASS |
| `dock-price-checkout` | PASS as implemented; the test checks only copy and the `href` |
| `phone-qr-scanning` | PASS with the recorded camera/decoder fixture |

The full browser suite also passed: 40/40 tests. The claims inventory is still
release-blocking because prominent server/team/backup/read-only promises have
no claim entries, and `dock-price-checkout` does not request the checkout URL.
That URL returns 404 in production.

### Cold first read — PASS

A fresh, signed-out desktop and 390 px mobile visit answers all three questions
without scrolling:

- What: “Check deliveries against the purchase order.”
- For whom: “For small receiving teams that need a clear record before the supplier van leaves.”
- First action: “Try it with sample data”, beside “Opens one ready PO. No account.”

One click opens populated PO NB-1047 and a persistent “Demo — sample data,
nothing is saved” banner with reset and real-workspace actions. Evidence:
`evidence-verification-3/live-first-read-desktop.png` and
`live-first-read-mobile.png`.

### Deployment identity — PASS

- Live `/health` and `/ready` return the full candidate SHA
  `ee012af218c7a77894ef11997f60f8c261c29943`.
- The SHA-stamped local and live `index.html` have the same SHA-256
  `3aa633f635f4864ff0fc92a980e1b14c7aacbe3037ba714d7ed76ea9b047603a`.
- The local and live main JS have the same SHA-256
  `18253249571f35add005a1552f122658432f60406582d51aa7209ab28638148a`.
- The footer shows `Build ee012af218c7`.

This is fresh evidence; the result is not attributed to a stale deployment.

## Release-blocking defects

### Critical — tenant records can overwrite another customer's data

Purchase orders use a globally unique `id` primary key, although the browser
derives the ID only from the PO number. Common values such as `PO-1001` therefore
collide across customers. The upsert on conflict updates the existing row's
payload, supplier, and number without changing or checking its tenant. Tenant B
can overwrite Tenant A's PO payload, after which Tenant A reads Tenant B's data.

The same pattern exists for draft receipt IDs: lookup and conflict update are by
global receipt ID rather than `(tenant_id, id)`. This violates the required
per-user isolation and makes the paid backend unsafe for real supplier data.
Evidence: `api/src/lib.rs:185-186`, `api/src/lib.rs:444`,
`api/src/lib.rs:520-534`, and deterministic PO IDs at
`src/lib/domain/importCsv.ts:59-63`.

### Critical — Dock cannot be purchased or used as the promised team product

Fresh GET and HEAD requests to the exact production link behind “Start Dock
checkout” returned HTTP 404 with:

```json
{"error":"enabled factory product","status":404}
```

The page says `$49 USD per site each month`, while the supplied acceptance brief
specifies `$149/site/month`. No accepted change to that contract was supplied.
The checkout return form is also incomplete: opening `/?license=qa-return-token`
left the token in the URL and stored nothing.

After sign-in, the client only pushes local IndexedDB records to the API. All
workspace loads still call `workspaceRepository`; there is no client call to
the server PO list or get routes, and the API has no GET receipt, event, or
attachment route. A fresh device or cleared browser therefore cannot open the
records advertised as server-owned. Refresh also clears the in-memory access
token and member/site state; the app restores auth only on the callback route.

Membership does not repair this. `/me` provisions a new tenant and site from
every user's own `oid`. `allowed_site` may find a shared site but returns the
new user's personal tenant, so subsequent list queries cannot return the shared
tenant's records. There is no member-management UI or sign-out action.

Evidence: `src/App.svelte:90-103`, `src/App.svelte:150-170`,
`src/App.svelte:208-219`, `api/src/lib.rs:327-359`, and the API route inventory
at `api/src/lib.rs:865-877`.

### Critical — the server audit and retention promises are not trustworthy

- Every server timestamp is the constant `2026-08-28T00:00:00Z`; receipts,
  events, entitlements, and attachments created later are permanently
  misdated (`api/src/lib.rs:138-140`).
- The browser's “Record a correction” changes only local IndexedDB. Sync sends
  the finalized receipt to the normal save endpoint, which rejects it; there
  is no server correction-event endpoint. HTTP failure statuses are never
  checked, so the UI still reports success.
- Event insertion, transaction commit, and backup copy errors are discarded,
  yet finalization returns 200. The backup is a plain copy of the SQLite main
  file while WAL mode is enabled, so the just-committed WAL data is not proven
  to be in the copied snapshot.
- Attachments can be uploaded but have no retrieval route. Durable `/data`
  mounting remains an unverified operator action in the prior handoff.

These defects contradict the immutable audit trail, evidence retention, team
inbox, and backup statements on the landing/privacy/README pages.

### High — billing status is recorded but never enforced

`/billing/attach` writes an entitlement status, but no read or write route
consults that table. An unlicensed, invalid, expired, or revoked account can use
the same write endpoints as an active account. The terms promise that a lapsed
site becomes read-only and retains export; neither behavior exists. The return
license is not automatically stored or verified, and a manual invalid token is
stored locally before verification.

### High — the service worker can cache authenticated API responses across users

The service worker intercepts every same-origin GET except `/health`, performs
cache-first matching by URL with `ignoreVary: true`, and caches every successful
response. This includes `/api/v1/me` and the PO list endpoint. The cache name is
global to the build, not to the signed-in user or tenant. It can therefore serve
one user's cached profile/site or future PO response to another user of the same
browser profile, and it overrides the server's `no-store` header.

Evidence: `public/sw.js:22-34`.

### High — unlisted and unproved claims fail the claims contract

There is no claim/test for these public promises:

- “Dock keeps a team inbox, receipts, evidence, and audit history on the server.”
- “It keeps a backup snapshot after finalization.”
- “A canceled or revoked subscription becomes read-only; export remains available.”
- Signed-in server retention and tenant-scoped object storage in `/privacy` and README.

The listed checkout test asserts a literal link but not a successful checkout,
so it passed while the live link returned 404. The server claims are also false
for the reasons above.

### High — authentication does not meet the backend contract

The sign-in redirect correctly uses Microsoft Entra External ID at
`sociobotcustomers.ciamlogin.com`, tenant
`35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
`25c704f4-465a-47af-80ab-2c489466b697`, PKCE, and the production callback.
However:

- Live unauthenticated `/api/v1/me` returns 401 without the mandatory
  `WWW-Authenticate: Bearer` header.
- Discovery and JWKS are downloaded on every authenticated request instead of
  being cached for one hour.
- The UI offers no sign-out and does not restore an existing session on reload.

A real account was not available, so a complete login was not performed.

## Other defects

### Medium — zero quantities pass import validation

A negative quantity is rejected, but both `ordered=0` and a case
`units_per_case=0` imported successfully and produced a zero-expected receipt.
This is invalid integration data for the receiving workflow and should be
rejected with the existing row-specific recovery message.

### Medium — controlled PWA navigation masks real 404 responses

A cold direct request to an unknown route correctly returns HTTP 404. After the
service worker controls the page, it always returns cached `/index.html` for
navigation, so Playwright observed HTTP 200 for the same unknown route. The
designed not-found screen still renders, but the status contract is lost.

### Medium — route metadata is duplicated

Every rendered page contains two canonical links and two descriptions: the
static home tags plus route-specific Svelte tags. On `/privacy`, for example,
both `/` and `/privacy` are canonical. Route titles themselves are correct.

### Low — one moderate axe landmark finding

The landing manifest `<aside>` is nested in the hero section and triggers
`landmark-complementary-is-top-level`. There were no serious or critical axe
findings in light or dark mode on desktop or phone.

## Verified behavior that passes

- Live demo: unknown-code recovery, impossible-damage validation with focused
  alert, finalization, linked local correction, four-row CSV, reset, exact
  decimal math, and same-origin-only traffic.
- Real local workspace: missing/malformed CSV recovery, exact case conversion,
  MIME/signature evidence rejection, valid PNG retention, two retained POs,
  finalized receipt reopening after reload, and separate workspace IndexedDB.
- QR/barcode camera fixture and keyboard/typed fallback pass at both sizes.
- Offline demo reload preserved changed counts. Service-worker update left one
  active cache: `intake-desk:demo-shell:ee012af218c7a778`.
- Reduced-motion phone context had zero running animations and 1 ms maximum
  transition duration. Dark-mode axe checks passed.
- 320, 390, 640, and 1440 px checks had no horizontal overflow or controls
  below 44 px. The skip link has a visible 3 px focus outline; dialogs return
  focus on Escape.
- Initial and full demo request logs were same-origin only. No analytics,
  third-party runtime assets, AI calls, or raw provider secrets were found.
- Live headers include CSP, HSTS, `nosniff`, strict referrer policy, and camera-
  only permissions. HTML and service worker are no-store; hashed assets/fonts
  are immutable for one year; direct unknown routes return 404.
- Production rate limiting is active. A single HTTP/2 client sent 500 requests
  in 130 ms to `/api/v1/me`: 41 returned 401 and 459 returned 429. The observed
  allowance is a 40-request burst with 20 requests/second refill; every 429 had
  `Retry-After: 0`. `/health` and `/ready` are intentionally exempt.
- The release binary started with `env -i PORT=18080`, logged generated/default
  configuration, served the candidate SHA, reported ready, and shut down
  gracefully.

## Local gates

| Gate | Result |
| --- | --- |
| `npm ci` | PASS; 86 packages, 0 vulnerabilities |
| `npm run check` | PASS; 0 errors, 0 warnings |
| `npm test` | PASS; 9 files/16 web tests and 7 Rust tests |
| Every claims command separately | PASS; 16 commands, 32 project runs |
| `npm run test:e2e` | PASS; 40/40 |
| `npm run build` | PASS; `dist/` and release API binary |
| Candidate-stamped `BUILD_SHA=ee012af… npm run build` | PASS |
| `cargo fmt --manifest-path api/Cargo.toml -- --check` | PASS |
| `cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings` | PASS |
| `/opt/fleet/lib/verify-url.sh` | PASS after supplying its required evidence directory |

Docker is not installed in this worker, so the image itself could not be built.
Static inspection found the required multi-stage build, `rust:1-slim`, build
args, non-root distroless runtime, and `/data` directory.

## Performance and budgets

Fresh mobile Lighthouse production run:

| Category/metric | Result |
| --- | ---: |
| Performance | 94 |
| Accessibility | 100 |
| Best practices | 100 |
| SEO | 100 |
| LCP | 2.24 s |
| FCP | 1.65 s |
| TBT | 220 ms |
| CLS | 0 |

Build sizes: initial JS 102.26 KB raw / 34.48 KB gzip; deferred MSAL chunk
240.54 KB raw / 60.82 KB gzip; CSS 22.08 KB raw / 5.04 KB gzip; three fonts
111.33 KB total. The individual JS/CSS/font budgets pass.

## Evidence

Evidence is under `.factory/evidence-verification-3/`, including first-read,
finalized receipt, mobile offline/reduced-motion, dark mode, responsive
screenshots, live headers/bodies, exact live assets, checkout 404, verify-url
output, and `lighthouse-live.json`.

## Required next verification

Do not release until the checkout is registered and tested end to end; tenant
keys and upserts are scoped safely; a second signed-in user can load the same
site on a fresh device; server receipts/events/evidence can be retrieved;
corrections, timestamps, backups, and entitlement enforcement are durable;
authenticated API responses bypass public caches; auth responses/session/sign-
out meet the CIAM contract; public claims have sandbox tests; and the supplied
price contract is resolved explicitly.
