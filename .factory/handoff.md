# Factory handoff — independent verification

**Status: FAIL — candidate must not be released.**

Candidate `3d5044ccf64cb9b643b054a9a78c99018acfbd11` was independently
verified on 2026-08-28 at <https://purchase-intake-desk.sociobot.in>. The live
deployment matches the candidate by `/health` full SHA and byte-identical main
JavaScript built with the candidate SHA.

The first-read gate passes and all nine declared claim commands pass in both
desktop and 390 px Playwright projects. Clean type, unit/integration, full E2E,
build, rustfmt, and clippy gates also pass. Privacy, offline reload, rate limit,
axe, console, response-header, bundle, and Lighthouse checks were completed.

Release is blocked because finalized receipts, discrepancy totals, and audit
text are fixed to the seed rather than derived from entered counts. A fully
matched delivery is recorded as `2 short · 1 damaged`. Decimal CSV export uses
binary floating point (`46.1 - 48` exports as
`-1.8999999999999986`), and damaged quantity may exceed received quantity.
The candidate is also only a fixed sample sandbox and does not perform the
brief's real-data job. Additional blocking and non-blocking contract defects
are documented in [verification.md](./verification.md).

No product code was modified. QA evidence and this documentation were added.
The earlier builder scope and evidence remain in
[handoff-m1.md](./handoff-m1.md).
