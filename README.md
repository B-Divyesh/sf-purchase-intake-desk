# Intake Desk

Intake Desk helps small receiving teams check supplier deliveries against a
purchase order. The M1 release is a complete local sample: count a delivery,
record shortages and damage, finalize an append-only receipt, add a correction,
and export receipt CSV.

Production URL: <https://purchase-intake-desk.sociobot.in>

One-click demo: <https://purchase-intake-desk.sociobot.in/?demo=1>

## Who it is for

Intake Desk is for 10–100-person distributors, workshops, and manufacturers
that receive daily supplier deliveries but do not need a full ERP.

## What M1 includes

- A seeded West Yard purchase order, NB-1047, with three realistic lines.
- Typed and keyboard-scanner item lookup with a camera-denied fallback.
- Exact case-to-each quantity handling, shortage and damage classification.
- Local finalization, hash-linked correction history, and receipt CSV export.
- An isolated IndexedDB demo that works after a network drop.
- Responsive day/night design, keyboard operation, and public legal pages.

M1 is an evaluation sandbox. It does not include accounts, server-side customer
records, supplier messages, or checkout. Those are deliberately M2/M3 work in
[the venture plan](.factory/plan.md). The Dock plan will cost $149 USD per site
each month when production accounts open.

## Run locally

Requirements: Node.js 22+, npm 10+, and stable Rust.

```sh
npm ci
npm run dev
```

Open <http://localhost:5173/?demo=1>. The first visit installs the offline
shell. `Reset demo` deletes only `intake-desk:demo:v1` and restores the sample.

Run the Rust container service against a production web build:

```sh
npm run build:web
npm run dev:api
```

It listens on `PORT` or 8080 and exposes `/health`. Every non-health request is
limited by client IP, using the first valid `X-Forwarded-For` address.

## Test and build

```sh
npm run check
npm test
npm run test:e2e
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
```

Playwright 1.58.2 runs every claim in Chromium at desktop and 390×844. Each
test starts with a fresh browser context and the shipped sample. The build puts
the web artifact in `dist/` and creates the release Rust binary.

## Container

The image builds without `.git`, runs as a non-root user, and starts with only
`PORT` set.

```sh
docker build --build-arg BUILD_SHA=local -t intake-desk .
docker run --rm -p 8080:8080 intake-desk
curl http://localhost:8080/health
```

The factory owns deployment, DNS, identity callback registration, durable
storage, and the Dodo-backed Sociobot subscription registration. No secret is
stored in this repository or sent by the M1 demo.

## Project records

- [Milestone plan](.factory/plan.md)
- [Design system](.factory/design.md)
- [Claim tests](.factory/claims.json)
- [Demo contract](.factory/demo.md)
- [M1 handoff](.factory/handoff-m1.md)

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
