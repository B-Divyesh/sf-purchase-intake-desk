# Factory handoff — independent verification 5

Date: 2026-09-05

Controller stage: `m1-building`

## Verdict

**FAIL — 1 high-severity finding and 1 untested claim.**

Implementation `64df103df5f62e428da00cd4e25d6e3f47e6cae1` passes the M1
receiving job, repair regressions, local gates, and live phone/desktop checks.
The public `hosted-entra-session` claim remains untested because this worker had
no isolated CIAM account and no callback-registration confirmation. See
[verification-5.md](./verification-5.md).

No product code was changed. This handoff, the verification report, and new QA
evidence are the only repository changes.

## Candidate and live identity

- Implementation: `64df103df5f62e428da00cd4e25d6e3f47e6cae1`
- Documentation repair: `ae9873e66ea093f3aa2bf58355fe6e352dbfcb53`
- Verification checkout before this report: `503d8d967e64768f8e0dfea32c5b291080358f99`
- Live URL: <https://purchase-intake-desk.sociobot.in>

Live `/health`, `/ready`, and the footer currently report `503d8d9`. The diff
from `64df103` to `503d8d9` is documentation/report-only. The live main bundle
matches the candidate after normalizing only its displayed build stamp and
source-map filename, and live readiness proves the repaired path is present.

## Verification summary

- Clean install: 86 packages, 0 vulnerabilities.
- `npm run check`: 0 errors and 0 warnings.
- `npm test`: 17 web and 14 Rust tests passed.
- `npm run test:e2e`: 44 passed; 2 hosted-CIAM project runs skipped.
- Every claim command: 22 claims passed; `hosted-entra-session` skipped and is
  counted as untested.
- `npm run build`, Rust format, and clippy with denied warnings passed.
- Fresh-volume process start, readiness, clean shutdown, and same-directory
  restart passed. Tenant isolation, record reopen, audit/evidence/backup, and
  entitlement tests passed.
- Live desktop and phone first screens state the job, audience, and sample
  action before scrolling. The populated sample, persistent banner, reset,
  workspace separation, invalid paths, offline reload, keyboard/focus, 320 px,
  200% text, dark mode, reduced motion, legal routes, links, and designed 404
  passed.
- Live `/health` and `/ready` return 200. A protected API burst produced
  24×429 with `Retry-After`; admission recovered after two seconds.
- Live Lighthouse: performance 98, accessibility 100, best practices 100, SEO
  100; LCP 2.14 s, CLS 0, TBT 29 ms.

Evidence is in `.factory/evidence-verification-5/`.

## Needs operator action

Provide only an isolated product CIAM test account in `ENTRA_E2E_USERNAME` and
`ENTRA_E2E_PASSWORD`, and confirm registration of:

```text
https://purchase-intake-desk.sociobot.in/auth/callback
```

Then run:

```sh
ENTRA_E2E_USERNAME=... ENTRA_E2E_PASSWORD=... \
E2E_BASE_URL=https://purchase-intake-desk.sociobot.in \
npm run test:e2e -- --project=chromium --grep @claim:hosted-entra-session
```

Do not use a shared production credential. PASS requires this claim to complete
sign-in, callback, restored session, and sign-out.

Sociobot recurring-product mapping is also incomplete, but checkout is
deliberately disabled and accurately labeled. It is not an M1 defect.

## Milestone boundary

The current public milestone is M1. Supplier email, accounting/ERP sync,
notifications, operational administration, and later growth work remain future
milestones and were not assessed as shipped capabilities.
