# Intake Desk

Intake Desk helps small receiving teams check supplier deliveries against a
purchase order. Import a supplier CSV, count the delivery, attach evidence,
record discrepancies, and export a receipt.

Production URL: <https://purchase-intake-desk.sociobot.in>

One-click demo: <https://purchase-intake-desk.sociobot.in/?demo=1>

Real workspace: <https://purchase-intake-desk.sociobot.in/start>

## Who it is for

Intake Desk is for 10–100-person distributors, workshops, and manufacturers
that receive daily supplier deliveries but do not need a full ERP.

## What it includes

- Keep an inbox of imported purchase orders; later imports do not replace earlier receipts.
- Attach JPEG, PNG, WebP, or PDF evidence to its receipt.
- Compare expected and received quantities with exact decimal math.
- Finalize a hash-linked receipt, add corrections, and export CSV.
- Try the full flow with an isolated West Yard sample.

The no-account workspace is local to one browser. An entitled Dock site uses
Sociobot Entra sign-in and tenant-scoped SQLite storage under `/data`.
Finalized receipts, corrections, and evidence survive a server restart. A site
without an active entitlement is read-only on the server, while local CSV
export remains available.

The planned Dock price is **$149 USD per site each month**. Checkout is
currently unavailable because its Sociobot product mapping is not complete.
The site does not offer a broken payment link or accept a license meanwhile.

## Run locally

Requirements: Node.js 22+, npm 10+, and stable Rust.

```sh
npm ci
npm run dev
```

Open <http://localhost:5173/?demo=1>. The first visit installs the offline
shell. `Reset demo` deletes only `intake-desk:demo:v1` and restores the sample.
Open <http://localhost:5173/start> to import the CSV template or your own file.

Run the Rust container service against a production web build:

```sh
npm run build:web
npm run dev:api
```

It listens on `PORT` or 8080 and exposes `/health` and `/ready`. It boots with
only `PORT`; it creates `/data/intake-desk-rollback.sqlite3` (or `./data`
locally) with SQLite's `DELETE` rollback journal. This is the active database
for the one-replica mounted deployment. It also creates `/data/objects` for
evidence and a consistent `backup-latest.sqlite3` snapshot after each server
receipt finalization. `/ready` checks the open database can take a write lock
and that the evidence directory is writable. Set `DATA_DIR` only to override
the durable mount.

## Test and build

```sh
npm run check
npm test
npm run test:e2e
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
```

Playwright 1.58.2 runs every listed claim at desktop and 390×844. The build
puts the web artifact in `dist/` and creates the release Rust binary.
Backend tests create isolated temporary tenants and do not use production data
or billing.

## Hosted Entra verification

The normal test suite has no identity credentials. The hosted sign-in claim is
therefore reported as skipped unless an operator supplies a dedicated product
test account. Do not use a shared production account.

The missing provider inputs are an account username in `ENTRA_E2E_USERNAME`,
its password in `ENTRA_E2E_PASSWORD`, and confirmation that
`https://purchase-intake-desk.sociobot.in/auth/callback` is registered on the
shared Sociobot Entra SPA. With those inputs, run:

```sh
ENTRA_E2E_USERNAME=... ENTRA_E2E_PASSWORD=... \
E2E_BASE_URL=https://purchase-intake-desk.sociobot.in \
npm run test:e2e -- --project=chromium --grep @claim:hosted-entra-session
```

The test uses a fresh Chromium profile, completes hosted sign-in, reloads to
restore the session, and signs out. It never writes credentials to this repo.

## Container

```sh
docker build --build-arg BUILD_SHA=local -t intake-desk .
docker run --rm -p 8080:8080 intake-desk
curl http://localhost:8080/health
```

The factory owns deployment, DNS, and future shared-account configuration.

## Project records

- [Milestone plan](.factory/plan.md)
- [Design system](.factory/design.md)
- [Claim tests](.factory/claims.json)
- [Demo contract](.factory/demo.md)
- [M1 handoff](.factory/handoff-m1.md)

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
