# Intake Desk venture plan

Status: **M1 PASS; M2 CORE REPAIRED — external identity and billing checks remain**

Product: `purchase-intake-desk` · Artifact: `web-with-backend`

Planning work order: `venture-purchase-intake-desk-plan`

Source of truth: [brief.json](./brief.json), this plan, and [design.md](./design.md)

Last updated: 2026-08-28

This is the execution contract for milestone builders. A builder must read this
file, the latest handoff, and every earlier milestone handoff before changing
code. Update milestone status and write `.factory/handoff-m<N>.md` after every
build. A milestone ships only after its review and polish loop passes.

## 1. Product requirements

### Customer and situation

Intake Desk serves 10–100-person distributors, workshops, and small
manufacturers. The buyer is an operations or purchasing lead. The daily users
are receiving staff, storekeepers, and accounts staff.

A supplier van is waiting. The receiver has a printed packing list, an emailed
purchase order, and perhaps a barcode scanner or phone. Today they count by
hand, write on paper, update a spreadsheet later, and chase the supplier from
email. A missed shortage or damaged item becomes an argument without evidence.
Broad ERPs ask this team to redesign the rest of the business before the dock
can record one dependable receipt.

### Promise

**Turn every supplier delivery into a checked, evidenced receipt before it
leaves the dock.**

### The three jobs the product must nail

1. **Check a delivery against its purchase order.** Find the PO, scan or type
   an item code, count in the supplier's unit, see the difference immediately,
   attach evidence, and finalize a receipt even when the network drops.
2. **Resolve a supplier exception.** Turn a short, extra, damaged, wrong, or
   partial delivery into one evidence record; notify the supplier; record each
   reply, correction, and resolution without rewriting history.
3. **Fit the tools already in use.** Validate purchase-order CSV before import,
   preserve the source row, and export predictable receipt/discrepancy CSV for
   accounting or a later ERP migration.

### Success and product instrumentation

The pilot succeeds when at least 95% of physical deliveries are finalized in
Intake Desk on the calendar day received, and the median time from arrival to
first discrepancy notification falls by 50% from the pilot's two-week
baseline. The site dashboard calculates these from receipt events; no
third-party analytics or session replay is permitted. Supporting measures are
import rejection rate, offline sync failure rate, finalized-receipt correction
rate, and notification delivery failure rate. Performance figures stay out of
marketing copy until a claim test measures them.

### Plans and billing

| Plan | Price | Intended use | Entitlement |
| --- | ---: | --- | --- |
| **Sandbox** | $0 | Anyone evaluating Intake Desk | Sample data only; no account; nothing persists to a production tenant |
| **Dock** | **$149 USD per site each month** | One receiving site and its staff | Production PO imports, receiving, evidence, discrepancy workflow, audit, exports, and member access |

Dock is a recurring Dodo subscription sold only through the Sociobot billing
API. Dodo/Sociobot is merchant of record. There is no direct Dodo SDK or secret
in this repository. Export, accessibility, correction, and account deletion
are never withheld for a lapsed site. A lapsed site becomes read-only and can
export its data. Do not invent a second paid plan until customer evidence
supports one; Sandbox and Dock are the named launch tiers.

### Deliberately out of scope

- General ledger, invoices payable, payment approvals, and three-way matching.
- Stock locations, picking, replenishment, forecasting, or a full WMS.
- Purchase requisitions, sourcing, supplier marketplace, or catalogue buying.
- Autonomous acceptance of a delivery, autonomous supplier messages, or
  machine-generated changes to finalized records.
- Custom EDI in the launch milestones. CSV and documented webhooks come first.
- Native mobile apps. The installable web app and keyboard-wedge scanners are
  the supported launch surfaces.
- Claims of legal/compliance certification. The audit trail is tamper-evident,
  not a qualified electronic signature.

## 2. Evidence and wedge

### Demand signals

| Date | Evidence | What it supports |
| --- | --- | --- |
| 2026-01-08 | [HN discussion/API record](https://hn.algolia.com/api/v1/items/46541949): a small team describes spreadsheet/email procurement, missed orders, lost documents, manual packing-list matching, and ERP cost/weight. | The problem is recurring, operational, and painful before a company is ready for ERP. |
| 2026-05-11 | [Dolibarr issue #38168](https://github.com/Dolibarr/Dolibarr/issues/38168): goods reception cannot populate items from a supplier order and can create only an empty reception. | Even an incumbent ERP can fail at the exact receiving handoff; dependable PO-to-receipt loading is a wedge. |

These are directional signals, not proof of willingness to pay. M1 is built to
make the workflow testable with dock workers; M2 is the paid pilot test.

### Why someone switches

The switch does not require replacing accounting or procurement. A lead can
export open POs from a spreadsheet in the morning, train a receiver from the
sample workflow, and produce a discrepancy packet on the same day. Intake Desk
starts at the receiving desk, treats exceptions as the output, and preserves a
receipt trail that can later move into an ERP. Its comparison set is paper plus
spreadsheets, not an ERP feature matrix.

### Evidence still required

- Observe five real receiving sessions across at least two businesses.
- Obtain three anonymized PO CSV files and record mapping variance.
- Measure the pilot baseline from arrival to supplier notification.
- Confirm whether $149/site/month is acceptable with five budget holders.
- Confirm the most common scanner hardware and browser combination.

These gaps are experiments in the risk register, not excuses to broaden M1.

## 3. Architecture

### Stack decision

| Layer | Choice | Reason |
| --- | --- | --- |
| Web | Svelte 5, Vite, strict TypeScript | Receiving has non-trivial reactive state and offline queues; Svelte keeps the mobile bundle and component overhead small. React's ecosystem is not needed. |
| Local data | IndexedDB through a small repository adapter; service worker cache | The dock must keep counting through a network loss. Demo and production use different database names. |
| API | Rust 2021, axum, tokio, serde, rust_decimal, tower-governor, tracing | Receipt invariants, decimal quantities, concurrent sync, and audit events benefit from typed server code. |
| Durable data | SQLite in WAL mode on `/data`, with reversible sqlx migrations | A single regional container handles launch load and must boot with only `PORT`. All tables remain tenant-keyed. Move to PostgreSQL only after the measured write profile requires multiple replicas. |
| Files | Content-addressed files under `/data/objects`, metadata and SHA-256 in SQLite | Keeps launch operations self-contained; permits deduplication, retention, checksum checks, and complete backup/export. |
| Deploy | One non-root Container Apps image; axum serves `/api/*` and Vite `dist/` | One origin makes auth, offline, CSP, and deployment easier. No infra, DNS, or billing changes belong in this repo. |

The performance budget is initial JS ≤ 200 KB gzip (target ≤ 150 KB), CSS ≤
50 KB, self-hosted fonts ≤ 120 KB, LCP < 2.5 s, INP < 200 ms, and CLS < 0.1
on a throttled mid-range phone. ES2022 is the browser target.

### Runtime shape

```text
Phone / scanner / desktop
  └─ Svelte PWA
       ├─ IndexedDB: cached POs + idempotent outbox
       ├─ service worker: versioned shell and demo sample
       └─ same-origin /api/v1
            └─ axum
                 ├─ Entra JWT validation + tenant authorization
                 ├─ receipt/import/discrepancy domain services
                 ├─ SQLite WAL + append-only audit events
                 ├─ /data/objects evidence files
                 └─ durable outbox workers

External, explicit only:
  Sociobot Entra CIAM · Sociobot billing API (Dodo-backed)
  Sociobot AI gateway (optional packing-list read in M5)
  transactional SMTP/relay configured by an operator
```

### Repository target layout

```text
src/                         Svelte web app
  lib/components/            shared UI primitives and product components
  lib/design/                tokens and theme
  lib/domain/                quantities, receipts, discrepancies, CSV
  lib/storage/               production/demo IndexedDB repositories
  routes/                    route modules and metadata
api/
  src/main.rs                config, listener, graceful shutdown
  src/routes/                thin HTTP handlers
  src/domain/                invariants and domain services
  src/db/                    repositories and migrations access
  migrations/                reversible numbered sqlx migrations
  tests/                     route and isolation integration tests
tests/e2e/                   Playwright claim tests
public/                      manifest, service worker inputs, owned assets
.factory/                    plan, design, claims, demo and handoffs
```

Milestone builders may refine folders without changing the boundaries:
components do not fetch, routes do not contain receipt math, and database rows
do not flow directly to the browser.

### Data model and ownership

All identifiers are server-generated UUIDv7 strings. Every tenant-owned table
contains `tenant_id`; site-owned rows also contain `site_id`. Every query takes
an authorization context and filters both. Foreign keys include tenant/site
columns where SQLite permits so cross-tenant references cannot be created by a
bug. Timestamps are UTC RFC 3339 at the API and integer microseconds in storage.

| Entity | Important fields | Ownership and invariant |
| --- | --- | --- |
| `users` | `oid`, display name, email snapshot, created_at | `oid` from Entra is the stable key; email is display-only. |
| `tenants` / `sites` | name, time zone, retention days, status | A subscription entitles sites; site time zone determines “same day.” |
| `memberships` | tenant, user, role (`owner`, `manager`, `receiver`, `viewer`) | Unique per tenant/user; authorization is server-side. |
| `suppliers` | supplier code, name, contact channels | Tenant-owned; codes unique within tenant. |
| `purchase_orders` | source key, supplier, order date, currency, status, source hash | Site-owned; an import with the same source key/hash is idempotent. |
| `po_lines` | source row, SKU, description, ordered decimal, order unit, base-unit ratio | Original text and normalized quantity are both preserved. |
| `unit_definitions` / `unit_conversions` | dimension, symbol, rational numerator/denominator | Conversions are exact rational values; cross-dimension conversion is rejected. |
| `receipts` | PO, delivery reference, received_at, state, revision, finalized_by | Drafts may change. Finalized receipts are never updated or deleted. |
| `receipt_lines` | PO line, received decimal, condition, note | Decimal/rational math only; no binary floating point. |
| `receipt_events` | receipt, sequence, type, actor, payload JSON, previous_hash, event_hash | Append-only hash chain. Corrections and reversals are new events. |
| `attachments` | object key, media type, bytes, checksum, caption, retention_until | Only approved image/PDF types; file and metadata share retention. |
| `discrepancy_cases` | receipt, kind, expected/received, state, owner, notified_at, resolved_at | One case can cover related lines; resolution also appends a receipt event. |
| `case_messages` | case, direction, channel, subject/body snapshot, delivery status | Immutable message evidence; secrets and access tokens are never stored here. |
| `import_runs` / `import_rows` | source checksum, mapping, outcome, normalized/error payload | Preview is immutable; commit refers to the accepted preview checksum. |
| `idempotency_keys` | tenant, actor/device, key, request hash, response code/body | Unique for write endpoint and key; a changed payload with reused key is `409`. |
| `outbox_jobs` | kind, payload, attempt, run_after, locked_at, last_error | Database-backed jobs; side effects occur only after the domain transaction. |
| `entitlements` | site, tier, license fingerprint, status, checked_at, expires_at | Full license token is encrypted at rest; fingerprint is safe for logs. |

No hard delete touches finalized receipt events. Tenant deletion first revokes
access, creates an owner export, then removes ordinary data after the stated
cooling-off/retention period. If a customer requests an exception to audit
retention, the UI explains exactly what can and cannot be removed.

### Critical domain rules

1. A receipt can finalize only against the exact PO revision shown to the
   receiver. A stale revision returns `409` with a refresh-and-review action.
2. Quantities use `rust_decimal` on the server and decimal strings in JSON.
   Unit conversions use reduced rational factors and round only for display.
3. Finalization is one database transaction: receipt lines, discrepancies,
   event, hashes, and outbox jobs either all commit or none do.
4. Offline writes carry a device-generated idempotency key and base revision.
   Replays return the original response. Conflicts never silently overwrite.
5. An attachment is not evidence until its checksum, size, media type, caption,
   actor, and event reference have been committed.
6. A correction names the previous event, states a reason, and appends new
   values. The earlier event remains visible and exportable.

### API contract by milestone

All responses use JSON except file/CSV downloads. Errors have
`{code, message, action, request_id, field_errors?}`. Writes require
`Idempotency-Key`. List endpoints use opaque cursor pagination. `/health` is
public and exempt from limits; all other endpoints are limited.

| Method and path | Purpose | First milestone |
| --- | --- | --- |
| `GET /health` | Liveness and build SHA, no dependency check | scaffold |
| `GET /ready` | SQLite/object path writable and migrations current | M2 |
| `GET /api/v1/me` | User, memberships, active site and entitlements | M2 |
| `GET /api/v1/sites/:site_id/purchase-orders` | Filter open POs | M2 |
| `GET /api/v1/purchase-orders/:id` | PO revision and lines | M2 |
| `POST /api/v1/receipts` | Create draft from PO revision | M2 |
| `PATCH /api/v1/receipts/:id` | Save draft line counts | M2 |
| `POST /api/v1/receipts/:id/finalize` | Finalize atomically and append audit | M2 |
| `POST /api/v1/receipts/:id/corrections` | Append correction; never update final | M2 |
| `POST /api/v1/billing/attach` | Verify and bind returned Sociobot license | M2 |
| `POST /api/v1/imports/preview` / `POST .../:id/commit` | Validate, map, then import CSV | M3 |
| `POST /api/v1/attachments` | Stream approved evidence with checksum/limits | M3 |
| `GET/PATCH /api/v1/discrepancies/:id` | Read case; append state transition | M3 |
| `POST /api/v1/discrepancies/:id/notify` | Queue explicit supplier email | M3 |
| `GET /api/v1/exports/receipts.csv` | Stable accounting export | M3 |
| `GET /api/v1/audit` | Site audit cursor and filters | M4 |
| `POST/DELETE /api/v1/memberships` | Owner-managed staff access | M4 |
| `POST /api/v1/exports/account` | Queue complete portable export | M4 |
| `POST /api/v1/webhooks` | Owner-created outbound integration | M5 |
| `POST /api/v1/documents/extract` | Optional reviewed packing-list extraction | M5 |

### Authentication and authorization

Production routes use the shared Sociobot Microsoft Entra External ID tenant:

- Tenant ID `35c6fe40-0ec0-46b6-98c6-213ad4de6650`.
- Authority `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`.
- SPA client ID `25c704f4-465a-47af-80ab-2c489466b697`.
- Redirect URI `https://purchase-intake-desk.sociobot.in/auth/callback`.
- Frontend uses `@azure/msal-browser`, authorization-code PKCE,
  `loginRedirect`, `acquireTokenSilent`, scopes `openid profile email`, and
  `sessionStorage`. Landing, legal pages, and demo remain public.
- API fetches OIDC discovery at startup, reads issuer and `jwks_uri`, caches
  JWKS for one hour, and validates RS256, `aud`, `tid`, `iss`, `exp`, and
  `nbf`. Invalid tokens return `401` plus `WWW-Authenticate: Bearer`.
- `oid`, never email, identifies a user. Roles are tenant memberships checked
  for every route. An object ID supplied by the browser never determines its
  tenant.
- Defaults are provided by `ENTRA_TENANT_ID`, `ENTRA_TENANT_SUBDOMAIN`, and
  `ENTRA_CLIENT_ID`; these optional environment variables can override them.

Operator action before M2 production verification: register the callback URI
above on the shared SPA application. The builder records confirmation or the
remaining action in the handoff.

### Billing

The public Dock action links to
`https://api.sociobot.in/api/v1/products/purchase-intake-desk/checkout`. Staging
uses the same path on `https://pilot-api.sociobot.in` and Dodo test card
`4242 4242 4242 4242`. The factory registers this product as the recurring
$149/site/month subscription; the app never calls Dodo directly.

After checkout, the frontend reads `?license=<token>`, stores it under
`sb_license:purchase-intake-desk`, immediately removes it from the URL, and
optimistically uses only a still-current cached verdict. It verifies through
the Sociobot `/verify?license=` endpoint no more than daily, provides “Have a
license? Paste it,” and quietly returns a lapsed site to read-only. After sign
in it also sends the token once to `/api/v1/billing/attach`; the API verifies it
independently, encrypts it with a first-boot `/data` key, and binds one active
site. Refund/revocation clears entitlement at the next verification. Billing
responses and tokens are never logged.

### Offline and sync model

- M1 uses `intake-desk:demo:v1`; production later uses
  `intake-desk:tenant:<tenant_id>:site:<site_id>:v1`. Code refuses to open a
  production store while the demo banner is active.
- The service worker precaches only versioned shell assets and the demo seed.
  Auth HTML and API responses are not placed in the Cache API.
- Production caches open assigned POs, their displayed revisions, draft
  receipts, thumbnails, and an outbox. It does not cache the entire audit log.
- Every local write has an idempotency key, base revision, queued time, and
  actor/device identifier. A visible connection strip distinguishes “saved on
  this device,” “syncing,” “synced,” and “needs review.”
- Sync runs on reconnect and explicit retry. Background Sync is an enhancement,
  never a requirement. Safari and unsupported camera browsers retain manual
  entry, file picker, and explicit retry.
- A remote change against the same PO pauses finalization. The user compares
  versions; the app never chooses silently.

### Scanners, units, and hardware fallback

Keyboard-wedge USB/Bluetooth scanners are the baseline and work through a
focused capture field. Camera scanning uses `BarcodeDetector` when present and
a lazily loaded, self-hosted decoder only if the eventual bundle stays within
budget. Every scan has a typed/manual path. M1 tests scanner bursts as keyboard
input, manual SKU entry, camera denial, narrow view, and reconnect. Supported
launch browsers are the latest two Chrome/Edge releases, Safari 17+, and
Firefox latest; camera scan is progressive enhancement.

### Files and retention

M3 accepts JPEG, PNG, WebP, HEIC after safe conversion, and PDF. The server
sniffs content, rejects executable/polyglot content, strips image metadata when
possible, limits a file to 10 MB and a receipt to 50 MB, uses generated object
names, and never serves uploads as active HTML. Download responses set
`Content-Disposition: attachment`, `X-Content-Type-Options: nosniff`, and a
strict content security policy.

Default retention is 24 months after receipt finalization and is configurable
from 90 days to 7 years. A daily job marks expired files, waits seven days,
then deletes file bytes and appends a metadata-only retention event. Legal and
privacy pages state the selected site policy in plain words.

### Background work and email

The SQLite outbox runs bounded workers in the API process. Jobs include
supplier email, entitlement recheck, demo/temporary cleanup, retention,
portable export, webhook delivery, and backup verification. Claims use
recorded mail/webhook fixtures; tests never send live messages.

Email is transactional and sent only when a user explicitly chooses “Notify
supplier.” The confirmation shows recipients, subject, body, and attachments.
The sender is a configured site identity through the factory relay; when no
relay is configured the UI downloads an `.eml` draft and marks it “not sent.”
There are no marketing lists or automatic supplier follow-ups.

### AI: one earned, optional use

No AI belongs in M1–M4. Structured CSV and manual receiving remain the reliable
paths. In M5, “Read packing list” may propose line codes and quantities from an
uploaded image/PDF. It calls only `https://api.sociobot.in/v1`, asks `/models`
for the first `gpt-5.6-*`, prefers `gpt-5.6-sol`, shows exactly what will be
sent, runs only after an explicit action, streams proposals, and requires a
human to accept every field. Undo returns to the unmodified document.

The backend uses `FACTORY_SOCIOBOT_KEY` only if present, with 5 actions/user/day
and a hard daily spend cap; otherwise the UI offers removable BYOK stored in
the browser. Demo and automated tests use a recorded response and spend
nothing. Product copy says “Read packing list,” never “AI-powered.”

### Rate limits and edge security

`tower-governor` keys client IP from the first valid `X-Forwarded-For` hop at
trusted factory ingress and otherwise uses the peer IP. Limits are also keyed
by user/site where authenticated. Every `429` includes `Retry-After`; health is
the only exemption.

| Class | Allowance |
| --- | --- |
| General API reads | 20 requests/second/IP, burst 40; 600/minute/user |
| Auth and billing attach/verify | 10/minute/IP; 6/minute/user |
| Receipt draft writes | 60/minute/user and 120/minute/site |
| Finalize/correct/notify | 20/minute/user and 60/minute/site |
| Import preview/commit | 5/minute/site |
| Attachment upload | 20/minute/site plus byte limits |
| Demo provisioning/reset | 10/minute/IP; ephemeral workspace TTL 24 hours if server-backed later |
| AI extraction | 5/day/user plus daily server spend cap |

The API validates sizes and enums at the edge, parameterizes every query, uses
same-origin CORS by default, caps request bodies, redacts PII/tokens from logs,
and returns request IDs. Response headers include CSP matching actual origins,
HSTS in production, `X-Content-Type-Options: nosniff`, `Referrer-Policy:
strict-origin-when-cross-origin`, and restrictive Permissions Policy. CSRF is
not applicable to bearer-auth API calls; the app never uses bearer tokens in
cookies.

### Configuration, health, and deployment

The container must start with only `PORT` (default `8080`). `DATA_DIR` defaults
to `/data`, `DATABASE_URL` defaults to SQLite within it, and a CSPRNG creates
missing application encryption material there. Optional integrations disable
cleanly when their variables are absent. Startup logs which settings were
generated or supplied without printing values.

`/health` returns `{status, build_sha}`. `/ready` checks migration state and
read/write access. `BUILD_SHA` is compiled from Docker build args
`BUILD_SHA`, `GIT_SHA`, or `SOURCE_COMMIT`, all defaulting to `dev`. The Docker
build never reads `.git`, uses a multi-stage build, runs non-root, handles
SIGTERM gracefully, and exposes 8080.

### Observability and reliability

- JSON logs include timestamp, level, request ID, route template, status,
  latency, tenant/site pseudonymous IDs, job kind, and build SHA. Never log
  bodies, filenames, email addresses, tokens, or document contents.
- `/metrics` is ingress-restricted and reports request/latency/error counts,
  SQLite busy time, outbox age/failures, sync conflicts, attachment bytes, and
  receipt finalization duration. Product metrics are aggregate site counts.
- Initial SLO: 99.5% monthly availability; 99% of non-upload API reads under
  500 ms; no acknowledged finalized receipt lost. Alert on 5-minute error rate
  > 2%, readiness failure, oldest outbox job > 10 minutes, or backup age > 26 h.
- Graceful shutdown stops accepting writes, drains in-flight requests and
  bounded jobs, checkpoints WAL, and exits within the platform grace period.

### Backup, recovery, and export

A daily online SQLite backup plus content-addressed object snapshot is written
to a separately mounted backup path when available, encrypted, checksummed,
and retained daily for 30 days and monthly for 12 months. With no backup mount,
readiness remains healthy but metrics and the admin screen show “backup target
not configured”; production acceptance requires the operator to configure and
verify it. M4 performs an automated weekly restore into a temporary database
and checks event chains and attachment hashes. Target RPO is 24 hours and RTO
is 4 hours at launch.

Every site can always export CSV for POs, receipts, lines, discrepancies, and
events. An owner export adds JSON, attachments, checksums, schema version, and
a README in a ZIP. Exports are rate-limited, expire after 24 hours, and are
deleted after download/expiry.

## 4. Design system

The full visual contract is [design.md](./design.md); implemented tokens live
in `src/lib/design/tokens.css`, and component behavior is in
[component-inventory.md](./component-inventory.md).

### Direction: dock-stamp constructivism

The interface borrows the useful grammar of a receiving dock: warm manifest
paper, black routing ink, safety orange, ruled counts, clipped labels, and
decisive inspection stamps. Asymmetric constructivist blocks make status and
sequence legible at a glance. It must not resemble a generic card dashboard,
warehouse cosplay, or distressed vintage decoration. Texture is sparse and
never behind task text.

### Core tokens

| Token group | Contract |
| --- | --- |
| Light palette | manifest `#F4E9CF`, paper `#FFF9E8`, ink `#182327`, muted ink `#526064`, safety `#B83A1B`, route blue `#1F4B57` |
| Dark palette | night `#11191C`, night paper `#1B272A`, chalk `#F8EFD9`, muted chalk `#BEC7C3`, safety `#FF724C`, route blue `#78B9C7` |
| Feedback | success `#2E6A4F`, warning `#8A5A00`, danger `#A52A2A` in light; lighter equivalents in dark; always paired with text/icon |
| Type | Self-hosted Barlow Condensed for display/labels; Atkinson Hyperlegible Next for body/data; tabular numerals for quantities |
| Scale | 12, 14, 16, 20, 28, 40, 56 px; body never below 16 px in task surfaces |
| Space | 4, 8, 12, 16, 24, 32, 48, 64, 96 px |
| Shape | 0, 2, 6 px radii; 2 px rules; clipped/chamfered status plates, never pill-heavy |
| Motion | 120/180/260 ms; a finalized stamp lands once from its source; reduced motion uses an instant border/opacity change |

All specified text/background pairs are at least 4.5:1. Focus uses a 3 px
route-blue ring plus 2 px offset. Touch targets are at least 44×44 px.

### Component set (20)

`SiteHeader`, `DemoBanner`, `DockButton`, `TextLink`, `Field`, `StatusStamp`,
`FilterRail`, `PurchaseOrderRow`, `POManifest`, `QuantityStepper`,
`ScanCapture`, `ConnectionStrip`, `AttachmentTray`, `DiscrepancyPanel`,
`ReceiptTimeline`, `ImportMapTable`, `ExportMenu`, `ConfirmDialog`,
`LiveNotice`, and `StatePanel`. Each has default, focus, disabled, loading,
error, offline, and empty behavior where applicable. The inventory defines
keyboard and narrow-screen rules.

### Five key screens in words

1. **Landing + live sample:** a left-weighted manifest headline and orange
   sample-data action sit beside one actual open-PO strip, followed by the live
   product, three verb steps, limits/privacy, Dock price, and footer.
2. **PO inbox:** dense ruled rows group “Due,” “Part received,” and “Closed.” A
   persistent scan/find field comes first. Status is word + stamp, never color
   alone. On phone the next delivery and search stay; secondary columns fold.
3. **Receive delivery:** expected lines form the fixed manifest edge; received
   counts are the dominant controls. Scanner state and connection state remain
   visible. Finalize is unavailable until differences are classified.
4. **Discrepancy evidence:** a safety-orange exception strip pairs the exact
   expected/received difference with photos, notes, notification preview, and
   an immutable event rail. It reads like a case file, not a chat app.
5. **Import/admin desk:** mapping occupies a wide ruled sheet with source,
   mapped field, sample, and error in one row. Settings, members, billing,
   retention, exports, and backup state are secondary tabs with real URLs.

### State, responsive, and accessibility rules

- Empty states name what belongs there and offer one verb action. Loading uses
  reserved ruled rows, not a spinner-only blank. Errors say what happened and
  one recovery step. Offline state says whether work is only on this device.
- At ≤ 639 px, one task occupies the screen; tables become labeled row pairs;
  secondary filters move into a sheet; the bottom action bar respects safe
  areas. At 640–1023 px, counts and manifest share 5/7 columns. At ≥ 1024 px,
  the audit/context rail appears without shrinking task text.
- Every route has one `<h1>`, ordered headings, a skip link, landmarks, and a
  route announcement. Navigation updates title, moves focus to the `<h1>`, and
  restores back/forward scroll. Dialogs trap and restore focus. Scanner input
  never creates a keyboard trap.
- Light and dark treatments follow the user setting. High contrast does not
  depend on texture. At 200% zoom, finalization and connection state remain in
  document flow. All motion stops or becomes instant under reduced motion.

### Site structure and copy

Public routes are `/`, `/demo`, `/privacy`, `/terms`, and `/404`. Product
routes appear only in their milestone. Every route has a ≤60-character plain
title, ≤155-character description, canonical link, one plain `<h1>`, and
product-owned Open Graph art. The header has wordmark, Demo, Pricing, and Sign
in (when implemented); the footer has the one-line description, Privacy,
Terms, Param Factory credit, and build ID.

M1 first-screen copy is fixed for implementation and copy audit:

- Headline: **“Check deliveries against the purchase order.”**
- Sentence: **“For small receiving teams that need a clear record before the
  supplier van leaves.”**
- Action: **“Try it with sample data.”** Adjacent note: “Opens one ready PO. No
  account.”
- Facts: “Keeps counts through a network drop.” “Exports receipt CSV.” “Dock
  plan: $149 per site each month.”

The M1 builder must extract all landing sentences into
`.factory/copy-audit.md`, count words, remove banned terms, and keep terminology
to `purchase order`, `receipt`, `discrepancy`, `site`, and `supplier`.

## 5. Milestones

Every milestone fits one focused builder session (target 3–4 hours), produces a
reviewable deployment candidate, and preserves `/?demo=1`. If scope pressure
appears, defer the listed non-core item; never weaken tenant isolation, audit
immutability, offline clarity, accessibility, or claim tests.

### M1 — See and complete the dock job in a sandbox

Status: **BUILT — repair 1 deployed and verified**

Goal: A visitor can understand Intake Desk and complete one realistic receipt
with a discrepancy, offline, without an account or server write.

Routes/screens added:

- `/` — standard landing skeleton plus a live PO preview.
- `/?demo=1` — immediately enters the seeded demo and canonicalizes to
  `/demo?demo=1` without reading production storage.
- `/demo` — seeded PO inbox and persistent demo banner.
- `/demo/purchase-orders/:po_id` — sample manifest.
- `/demo/receive/:po_id` — scan/type/count/condition flow.
- `/demo/receipts/:receipt_id` — finalized receipt and event history.
- `/demo/discrepancies/:case_id` — generated evidence record.
- `/privacy`, `/terms`, `/404` — complete, styled, linked pages.

Seed exactly this scenario so screenshots and tests remain stable:

- Site: **West Yard**; supplier: **Northline Bearings**; PO **NB-1047**.
- BRG-6204: 120 each expected and received.
- BLT-A42: 4 cases × 12 = 48 each expected; 46 each received (2 short).
- SEAL-28: 24 each expected; 24 received, 1 marked damaged.
- Packing list `NL-8821`; received 2026-08-28 09:40 local by demo user Sam.

Scope:

- Implement the visual system, responsive shell, metadata, owned SVG/social
  art, self-hosted fonts, manifest/PWA files, versioned service worker, and
  history routing with focus/title management.
- Implement a repository interface with demo-only IndexedDB database
  `intake-desk:demo:v1`; seed/reset is deterministic and separate from the
  future production adapter. Demo banner always shows “Demo — sample data,
  nothing is saved,” “Reset demo,” and “Start for real.”
- Implement decimal/rational quantity math, scan burst/manual SKU capture,
  expected-versus-received difference, condition, note, local sample evidence,
  classification, finalization, append-only hash-linked local events,
  correction event, discrepancy screen, and receipt CSV download.
- Disable camera-dependent claims; show camera denial/unsupported fallback to
  typed or keyboard-wedge entry. No fake server calls, sign-in, payment,
  supplier send, or production persistence.
- The sample and shell work after the first visit with the browser offline.
  The only allowed runtime request in the demo flow is same-origin static
  content. Leaving/resetting discards changed demo data.
- Landing pricing says Dock is $149/site/month and “Accounts open in the next
  milestone”; do not expose a dead checkout action.

Claims in `.factory/claims.json`:

`demo-entry-reset`, `expected-vs-received`, `exact-unit-conversion`,
`partial-discrepancy`, `immutable-correction`, `receipt-csv-export`,
`offline-reload`, `demo-isolation`, and `scanner-manual-fallback`.

Tests:

- Vitest: decimal/rational conversion, difference classification, event hash
  chain, CSV escaping/schema, seed/reset, demo repository prefix, route title.
- Playwright: exactly one test tagged for every M1 claim, all starting from a
  fresh context and `/?demo=1`; mobile 390×844 and keyboard-only receiving.
- Playwright + axe: no serious/critical issues on every route; focus returns
  from dialogs and route changes announce the new heading.
- Offline claim: first load online, wait for service worker, set context
  offline, reload finalized receipt, create another draft, then reconnect.
- Network privacy claim: record the whole demo flow and assert every request is
  same-origin and no production IndexedDB name opens.
- Build: `npm test`, `npm run check`, `npm run build`, bundle budget check, no
  console/page errors. `dist/` contains real 404, sitemap, robots, icons, and
  social image.

Definition of done:

- A stranger reaches used sample data in one click, records 120/46/24, marks
  damage, sees both discrepancy types, finalizes, corrects by new event, and
  downloads parseable CSV without instruction.
- All nine claims pass from clean demo state. The demo survives offline reload
  and never touches real data. Reset restores NB-1047 exactly.
- Landing follows the standard section order and fixed plain copy. Legal pages
  accurately describe local demo storage and planned account features.
- Phone, keyboard, empty, loading, invalid count, storage-full, camera-denied,
  offline, conflict-simulation, and 404 states are deliberate.
- Lighthouse mobile is ≥90 performance and ≥95 accessibility; initial JS/CSS/
  fonts and Web Vitals meet budgets. Results enter `.factory/handoff-m1.md`.
- Review → polish returns PASS before M2 starts.

### M2 — Sign in, pay, and use a real site

Status: **CORE BUILT AND REPAIRED — external identity and billing checks remain**

Goal: An owner can sign in, attach an active Dock subscription, create one site,
and repeat the M1 job against durable tenant-isolated records.

Routes/screens added:

- `/auth/callback`, `/start`, `/app/inbox`,
  `/app/purchase-orders/:po_id`, `/app/receive/:po_id`,
  `/app/receipts/:receipt_id`, `/app/settings/billing`.

Scope:

- Add reversible SQLite migrations for identity, tenant/site, membership,
  supplier, PO, receipt/event, idempotency, entitlement, and outbox tables.
- Implement Entra PKCE and backend validation exactly as specified above. Add
  owner/manager/receiver/viewer authorization tests and cross-tenant fixtures.
- Add production IndexedDB namespace, cached open POs, draft/outbox sync,
  idempotent writes, stale-revision review, and visible device/sync state.
- Wire Sociobot staging checkout and verification. Capture, strip, cache,
  restore, independently attach, encrypt, recheck, and revoke license. Support
  one Dock entitlement per site. Lapsed sites are read-only with export.
- Add real receipt APIs, append-only event hash chain, corrections, structured
  logs, limits, `/ready`, and migrations on startup. No uploads or email yet.
- Keep M1 demo byte-for-byte isolated and fully operational without auth.

Claims to add and test:

- “Signs in with Sociobot and signs out on this device.”
- “Keeps each site's purchase orders and receipts separate.”
- “Queues a receipt offline and syncs it once without duplication.”
- “Finalized receipts cannot be rewritten.”
- “Dock costs $149 per site each month through Sociobot checkout.”
- “A lapsed site can read and export its existing records.”
- “Rate-limited endpoints return 429 and Retry-After.”

Tests:

- Rust unit/integration tests with temporary databases for migrations, receipt
  transaction, hash chain, decimals, roles, tenant isolation, idempotency,
  stale revisions, limits, and generated-default startup.
- MSAL/JWKS and billing tests use recorded signed fixtures and the pilot API
  contract; one operator/verifier pass uses the real shared tenant and staging
  checkout if registered. No CI call spends money.
- Playwright covers sign-in return fixture, onboarding, real-site workflow,
  offline queue/reconnect, two-tenant denial, restore license, and lapse.
- `npm test`, Rust tests/clippy/fmt, build, Docker build/start with only `PORT`,
  health/readiness, SIGTERM, and 100 rps read smoke.

Definition of done:

- A new customer can go from landing to active paid site and first durable
  receipt in under five minutes, excluding the hosted identity/payment screens.
- JWT checks, authorization, isolation, encrypted entitlement, idempotency,
  immutable audit, rate limits, generated config, and graceful shutdown are
  verified. No secrets appear in client bundles or logs.
- The production app works on a phone during a network loss and explains
  device-only versus synced state. Demo remains public and isolated.
- Redirect registration and Sociobot recurring product status are confirmed in
  handoff, or explicitly listed as operator action before production deploy.
- Review → polish returns PASS before M3 starts.

### M3 — Import POs and close the supplier loop

Status: **PLANNED — blocked on M2 PASS**

Goal: A site can bring in its actual open POs, attach evidence, notify a
supplier of a discrepancy, resolve it, and export accounting-ready records.

Routes/screens added:

- `/app/import`, `/app/import/:run_id`,
  `/app/discrepancies`, `/app/discrepancies/:case_id`,
  `/app/exports`, `/print/receipts/:receipt_id`.

Scope:

- CSV upload, encoding/delimiter detection, explicit mapping, sample preview,
  row errors, exact unit mapping, duplicate/source-hash detection, and atomic
  commit. Ship a documented template; never partially import invalid rows
  unless the user explicitly exports errors and chooses valid-only commit.
- Evidence upload pipeline and safe serving with all limits, checksums,
  retention metadata, phone capture/file picker, captions, and offline upload
  queue. Preserve original locally until server acknowledgement.
- Discrepancy kinds, assignment, evidence packet, explicit notification
  preview, transactional email or `.eml` fallback, delivery status, resolution
  and correction events. Never claim an unsent email was sent.
- Stable versioned receipt/discrepancy CSV, printable receipt/evidence view,
  and public documentation of columns, decimals, time zones, and schema version.

Claims to add and test:

- “Previews every CSV error before it changes purchase orders.”
- “Imports the same source file once.”
- “Keeps photo/PDF evidence with its receipt checksum.”
- “Shows recipients and evidence before notifying a supplier.”
- “Records whether the notice sent or only downloaded as a draft.”
- “Exports versioned receipt and discrepancy CSV without losing decimals.”

Tests:

- Fixture matrix for UTF-8/BOM, quoted commas, missing/extra headers, duplicate
  PO/line, invalid date/decimal/unit, 50k rows, and malicious formula cells.
- Attachment type sniffing, size/receipt caps, path traversal, checksum,
  retention, offline resume, and forbidden active content.
- Recorded mail success/failure, `.eml` fallback, event order, resolution,
  export formula neutralization and round-trip decimal tests.
- E2E runs an import → receive → discrepancy → notify/fallback → resolve →
  export flow at desktop and phone widths; all new claims have one tagged test.

Definition of done:

- A pilot can use its own CSV without developer mapping, see all rejected rows,
  receive a delivery, preserve evidence, and create an honest supplier notice.
- Accounting can consume the documented CSV; original imports and final receipt
  history remain traceable by source/checksum.
- File privacy, retention, limits, empty/error/loading/offline states, and
  notification failure recovery pass. Review → polish returns PASS.

### M4 — Operate the service without a developer

Status: **PLANNED — blocked on M3 PASS**

Goal: A site owner and service operator can manage access, retention, exports,
backups, jobs, and failures without direct database work.

Routes/screens added:

- `/app/audit`, `/app/settings/site`, `/app/settings/members`,
  `/app/settings/retention`, `/app/settings/integrations`,
  `/app/settings/account`, `/app/exports/:export_id`.

Scope:

- Member invite through Entra identity, role change, revoke, last-owner guard,
  session/access refresh, and actor display that survives member removal.
- Site time zone, supplier contacts, retention configuration, legal copy,
  account export/delete workflow, and read-only lapse controls.
- Searchable audit view with hash verification; complete owner ZIP export;
  deletion/retention jobs; download expiry; no global administrator browsing of
  customer documents.
- Operator-grade health/readiness/metrics, outbox retry/dead-letter view without
  document bodies, backup status, weekly restore verification, alert runbook,
  load smoke, and recovery drill.
- Transactional notifications for member invite, export ready, job failure to
  site owner, and impending retention deletion; all are necessary and opt-out
  where appropriate.

Claims to add and test:

- “Owners control who can receive, manage, or only view.”
- “Every finalized change names its actor and verifies its event chain.”
- “Owners can download a complete site export.”
- “Expired evidence follows the site's stated retention period.”
- “Backups are checked by restoring them each week.”
- “Health, rate, and job failures are visible without exposing document data.”

Tests:

- Role matrix, last-owner and removed-user tests; export/delete and retention
  fake-clock tests; chain tamper detection; download authorization/expiry.
- Backup a realistic database/object set, destroy a disposable copy, restore,
  compare row counts/hash chains/object checksums, and record duration.
- Metrics cardinality/redaction, job retry/dead-letter, readiness dependency,
  graceful shutdown, 100 rps smoke, and runbook tabletop.
- Browser tests for settings, audit, complete export, deletion confirmation,
  200% zoom, keyboard, and claim tags.

Definition of done:

- A customer administers its site and data without factory access. An operator
  can detect, triage, back up, and restore the service from documented steps.
- The handoff records measured RPO/RTO, restore evidence, load results, error
  budget alerts, known capacity boundary, and privacy checks.
- Review → polish returns PASS before M5 starts.

### M5 — Install, share, and connect without losing control

Status: **PLANNED — blocked on M4 PASS**

Goal: Make repeat dock use and supported integrations easier while preserving
the core audit and review boundaries.

Routes/screens added:

- `/app/settings/install`, `/app/settings/webhooks`,
  `/app/documents/:document_id/review`, `/developers/webhooks`.

Scope:

- Deliberate install prompt/help for supported browsers, installed launch to
  inbox, app badge where supported, update-available flow, and scanner/camera
  compatibility check. Never block use on installation.
- Signed outbound webhooks for receipt finalized, discrepancy opened/resolved,
  retry/backoff, replay UI, rotating secrets, documented JSON schemas and a
  local example consumer. No inbound writes in this milestone.
- Expiring, revocable, read-only evidence-link export for a discrepancy. It is
  an owner action, reveals only selected case fields, and is rate-limited and
  excluded from indexing. If secure external links cannot fit the session,
  ship downloadable evidence packets and defer links.
- Optional “Read packing list” proposal flow through the Sociobot gateway as
  specified above, with recorded demo, cost/privacy disclosure, manual review,
  undo, daily limits, and non-AI fallback. It never finalizes or notifies.
- Revisit SQLite from measured M4 load. Migrate to PostgreSQL only if sustained
  write lock or availability evidence crosses the documented threshold.

Claims to add and test:

- “Installs as a dock app and keeps the demo available offline.”
- “Sends signed, retryable receipt webhooks.”
- “Shares only the selected discrepancy and can revoke the link.”
- “Proposes packing-list fields for review; it never posts them itself.”
- “The full receiving workflow works without document extraction.”

Tests:

- PWA manifest/install/update/offline, compatibility fallbacks, and bundle
  budgets; webhook signature/retry/replay/rotation fixtures; share expiry,
  revocation, enumeration and indexing prevention.
- AI claim uses only a recorded gateway response in CI, verifies explicit send,
  streamed proposal, per-field accept, undo, redaction, cap/429, missing-key
  fallback, and zero demo spend. A verifier may run one live factory-key check.
- Full regression from demo and paid tenant; integration docs are executed from
  a fresh clone.

Definition of done:

- Repeat users can install the app, integrators can consume documented events,
  and a supplier can receive only intentionally shared evidence.
- AI remains optional, review-only, capped, and gateway-only. Removing every AI
  configuration leaves jobs 1–3 fully usable.
- Growth additions meet the same privacy, accessibility, rate-limit, offline,
  observability, and claim standards. Review → polish returns PASS.

## 6. Cross-milestone quality gate

No milestone is complete until all of the following are true:

- Every visible claim appears once in `.factory/claims.json` and has exactly
  one tagged observable sandbox test. Unprovable copy is removed.
- `npm test`, `npm run check`, and `npm run build` pass from a clean clone and
  create `dist/`; Rust fmt/clippy/tests and relevant Docker checks pass.
- There are no console/page errors, dead links, serious/critical axe findings,
  heading/title/landmark/alt/focus/contrast/motion failures, or 390 px overflow.
- The demo opens in one click from `/?demo=1`, uses sample data immediately,
  never reads/writes production storage, resets deterministically, and remains
  working through every later milestone.
- Empty, loading, invalid, error, permission, rate-limited, offline, reconnect,
  conflict, storage-full, and 404 states have a recovery action.
- Migrations are reversible; secrets are absent from repo/client/logs; every
  server route except health is limited; tenant isolation tests are negative as
  well as positive; export/delete/retention behavior matches legal copy.
- Metadata, canonical/OG/Twitter art, favicon/apple icon, sitemap, robots,
  security headers, build ID, privacy, terms, README, copy audit, demo doc, and
  milestone handoff match what actually shipped.
- Lighthouse mobile is ≥90 performance and ≥95 accessibility. Initial bundle,
  font/image, LCP/INP/CLS budgets pass and measured numbers are in the handoff.
- The independent review and subsequent polish report PASS before the next
  milestone begins.

## 7. Risks and experiments

| Risk / unknown | Consequence | Experiment that retires it | Decision threshold / owner milestone |
| --- | --- | --- | --- |
| Receivers will not stop using paper during unloading. | Adoption fails even if reporting is good. | Observe five sessions; M1 390 px usability test; timed paper versus demo task with keyboard scanner. | 4/5 complete without help and median entry no slower after second run; revise M1 interaction before M2. |
| Browser/camera/scanner combinations are unreliable. | Codes cannot be captured at the dock. | Test the top three pilot devices with USB/Bluetooth wedge, camera permission denied, low light, and manual fallback. | 100% can complete via wedge or manual; camera remains enhancement. M1/M2. |
| Offline queue duplicates or loses receipts. | Trust-destroying audit error. | Fault-injection tests at every write/response boundary plus 1,000 replayed idempotency keys. | Zero duplicate finalizations and all unresolved conflicts visible. M2. |
| Supplier unit conversions are ambiguous. | Wrong shortages or inventory handoff. | Collect three CSVs; build rational-unit fixture matrix; require preview for unmapped units. | No implicit cross-dimension mapping; 100% fixture precision. M1/M3. |
| CSV variance makes one-day onboarding unrealistic. | Customers need developer mapping. | Map three anonymized exports and run a non-developer preview/import test. | 80% rows map automatically and all remaining errors are actionable; otherwise add saved templates, not custom code. M3. |
| “Immutable” audit is misunderstood as certified compliance. | Legal/reputation risk. | Plain-copy review with two buyers; tamper test event chain; terms review. | Users describe it as correction history, not certification. M2/M4. |
| Evidence files exhaust disk or leak active content. | Outage or security incident. | Load 10k capped attachments, MIME/polyglot corpus, quota alert and retention deletion/restore tests. | No active render/path escape; alert before 70% capacity; documented capacity. M3/M4. |
| $149/site/month lacks willingness to pay. | No viable venture. | Five price interviews and three 30-day paid pilots, no bespoke discounting hidden from results. | At least 2/3 pilots renew or explicit value gap points to a scoped change. M2/M3. |
| Shared Entra callback or billing product is not registered. | Paid onboarding cannot complete. | Operator verifies callback and test recurring checkout before M2 review. | Real test identity and `4242` checkout return a valid entitlement. M2. |
| SQLite becomes the availability/write bottleneck. | Sync latency and single-replica ceiling. | M4 100 rps mix, busy-time metrics, measured peak ×10, restore drill. | Move to Postgres only if p99 >500 ms, busy errors occur, or multiple replicas become required. M4/M5. |
| Supplier email deliverability is poor or configuration absent. | Notification metric stays slow. | Recorded failure UX, configured pilot relay, delivery/bounce measurement, `.eml` fallback timing. | UI never claims sent without provider acceptance; fallback takes <2 minutes in pilot. M3. |
| Retention/deletion expectations conflict with audit history. | Privacy or contractual failure. | Ask pilot owner for policy; test export, cooling-off, attachment expiry, metadata-only event copy. | Signed pilot choice within 90 days–7 years and legal copy matches. M4. |
| Optional document extraction introduces incorrect counts or cost. | Unsafe receipts and margin loss. | 100-document labeled set, field-level acceptance tracking, fixed daily cap, compare time with manual entry. | Ship only if proposal precision ≥95% on key fields, every field remains reviewed, and time saved is meaningful. M5. |
| Visual system becomes decorative or hard to scan. | Dock users miss the primary state. | Two-second primary-action test, grayscale/color-blind review, 390 px screenshot review, five-user task test. | 5/5 identify next action/state; remove texture or asymmetry that interferes. M1. |

## 8. Planner handoff to M1

The repository currently contains tooling and a planning shell only. M1 owns
the first product implementation. It must preserve the selected stack, token
names, sample IDs/data, public copy, demo namespace, claim IDs, and milestone
boundary unless it documents new evidence and updates this plan first. It must
not begin real auth, production tenancy, billing, uploads, email, or AI.
