# Factory handoff — repair 2

## Status

Repair of verifier report `2f9ceacd7ad65d9105495224a53a59f7bbc2dc85`.

The decisive overwrite was reproduced before changing code: the old adapter
wrote every real PO to IndexedDB key `active-purchase-order`; importing PO-221
therefore replaced finalized PO-220. The adapter is now an IDB inbox keyed by
purchase-order ID, migrates the old single record once, and retains each local
receipt. Regression: `@claim:multi-po-retention` imports/finalizes PO-220,
imports PO-221, opens the inbox, then reopens the PO-220 receipt on desktop and
390 px mobile.

## What changed

- Added a server-owned SQLite/WAL model at `DATA_DIR/intake-desk.sqlite3` for
  tenants, sites, memberships, POs, receipts, append-only hash-chain events,
  attachment metadata, idempotency records, and entitlement fingerprints.
  Finalization writes a `backup-latest.sqlite3` snapshot; retained evidence is
  content-addressed under `DATA_DIR/objects`.
- Added protected `/api/v1` PO, receipt, finalization, evidence, membership,
  `/me`, and billing-attach routes. Every tenant query is membership scoped;
  the unit regression proves one user cannot access another site.
- Added Sociobot Entra External ID sign-in with MSAL PKCE/session storage and
  server verification of RS256 JWT signature/JWKS, discovery issuer, audience,
  tenant ID, expiry, and not-before. The stable `oid` is the server key.
- Replaced the permission probe with real `BarcodeDetector` QR/barcode camera
  scanning, an accessible live video dialog, and a keyboard/typed fallback.
- Replaced the unimplemented $149 copy with the controller-approved **$49 per
  site/month** recurring Dock checkout link, local restore field, server-side
  Sociobot verification, and entitlement fingerprint storage. The researched
  brief remains unchanged; its historic $149 hypothesis is not product copy.
- Updated privacy, terms, README, Docker persistent `/data` configuration,
  CSP, copy audit, and claims. Demo remains isolated and sends no auth,
  billing, or API requests.

## Verification

Run from a clean checkout:

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
```

Completed locally during this repair: type check; 16 web unit tests; 7 Rust
unit/integration tests; formatter; clippy; production build; all 40 Playwright
desktop/390 px tests; the added regressions for multi-PO retention, $49
checkout/restore, and fixture-backed phone QR scanning; and a release-binary
`PORT`-only `/health` + `/ready` smoke test. The initial application bundle is 34.47 KB gzip; the MSAL
chunk is loaded only after pressing Sign in (60.82 KB gzip).

Run the production service locally with only `PORT` (it falls back to `./data`
if `/data` is not writable):

```sh
npm run build
PORT=8080 STATIC_DIR=dist api/target/release/intake-desk-api
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

## Needs operator action

- Confirm/register `https://purchase-intake-desk.sociobot.in/auth/callback` on
  the shared CIAM SPA application `25c704f4-465a-47af-80ab-2c489466b697`.
  The redirect cannot be verified without a real tenant member.
- Register/confirm the recurring `$49` product configuration for
  `purchase-intake-desk` in Sociobot billing. The app uses only Sociobot hosted
  checkout and verification; it has no payment-provider secret.
- Mount durable `/data` storage for the container. Container Apps' default
  ephemeral filesystem is unsuitable for the database, object retention, and
  backup snapshot; verify the deployment has a persistent volume before
  treating Dock records as retained.
