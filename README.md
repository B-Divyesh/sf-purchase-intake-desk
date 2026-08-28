# Intake Desk

Intake Desk is planned for small receiving teams that check supplier
deliveries against purchase orders without adopting a full ERP. It will record
counts, evidence, partial deliveries, discrepancies, and portable CSV exports.

This repository is currently a **planning and tooling scaffold**. The product
workflow, demo, accounts, storage, and billing have not been built. M1 is
specified in [`.factory/plan.md`](.factory/plan.md) and its testable claims are
in [`.factory/claims.json`](.factory/claims.json).

Production URL: <https://purchase-intake-desk.sociobot.in>

Demo URL from M1: <https://purchase-intake-desk.sociobot.in/?demo=1>

## Who it is for

The intended customers are 10–100-person distributors, workshops, and small
manufacturers. Receiving staff need a dependable record while a delivery is
still at the dock. Operations and accounts staff need clear exceptions and an
export they can use in existing tools.

## Repository map

- `src/` — Svelte 5 planning shell and design-system starting points.
- `api/` — Rust/axum health and static-serving scaffold. Product APIs begin in
  M2.
- `.factory/plan.md` — PRD, evidence, architecture, milestones, tests, and
  risks.
- `.factory/design.md` — dock-stamp constructivism visual contract.
- `.factory/component-inventory.md` — 20 component contracts and states.
- `.factory/claims.json` — claims the M1 builder must implement and prove.
- `.factory/demo.md` — deterministic M1 sandbox and isolation contract.

## Develop

Requirements: Node.js 22+, npm 10+, and stable Rust.

```sh
npm ci
npm run dev       # planning shell at http://localhost:5173
npm run dev:api   # axum on PORT, default http://localhost:8080
```

Build the web app before starting axum if you want it to serve the static
shell:

```sh
npm run build:web
npm run dev:api
```

## Test and build

```sh
npm run check
npm test          # Vitest plus Rust tests
npm run build     # web artifact in dist/ plus release API binary
```

`npm run test:e2e` is reserved for M1 claim tests. Playwright is pinned to
1.58.2 to match the factory browsers; the planning scaffold intentionally has
no product E2E tests yet.

CI runs type checks, Rust formatting/clippy, unit/API tests, and both builds on
pushes and pull requests to `main`.

## Container

The multi-stage image builds the web shell and Rust server, runs non-root, and
serves on `PORT` (default `8080`):

```sh
docker build --build-arg BUILD_SHA=local -t intake-desk .
docker run --rm -p 8080:8080 intake-desk
curl http://localhost:8080/health
```

The factory owns deployment, identity callback registration, persistent
storage, secrets, DNS, and the Dodo-backed Sociobot subscription registration.
Do not change those from this repository.

## Privacy and billing direction

The M1 demo will use a separate local browser database and make no external
product-data requests. M2 will use the shared Sociobot Entra CIAM tenant and
the Sociobot billing API; the app will never handle passwords or call Dodo
directly. Full retention, export, deletion, `/privacy`, and `/terms` behavior is
specified milestone by milestone in the plan.

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
