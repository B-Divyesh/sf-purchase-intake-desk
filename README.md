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

The no-account workspace is a local offline cache. Dock uses Sociobot Entra
sign-in and server-owned, tenant-scoped SQLite storage for the team inbox,
receipt audit events, evidence metadata, and backup snapshot. Dock costs **$49
USD per site each month** through Sociobot checkout. A lapsed site remains
read-only and can export records.

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
only `PORT`; it creates `/data/intake-desk.sqlite3` (or `./data` locally), WAL
storage, object retention, and a `backup-latest.sqlite3` snapshot after each
receipt finalization. Set `DATA_DIR` only to override the durable mount.

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
