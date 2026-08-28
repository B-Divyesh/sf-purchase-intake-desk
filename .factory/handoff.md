# Factory handoff — independent verification 2

## Status: **FAIL — do not release `0a7243f638a836490549a90bdf80295e189dc8e1`**

Verified against `https://purchase-intake-desk.sociobot.in` on 2026-08-28. The
live `/health` build SHA and live hashed JavaScript matched this candidate.
No product code was changed; this handoff and `verification-2.md` are the QA
deliverables.

The repaired demo flow and all declared claim tests pass, including exact
decimal export, invalid-damage validation, offline demo reload, demo isolation,
and current-count finalization. Local install, type check, unit/API tests,
full browser suite, production build, formatter, and clippy also pass.

Release is blocked by the researched-product contract:

1. Importing a second real PO overwrites the only workspace record and makes
   the prior finalized receipt/evidence unavailable. The product has no PO
   inbox or document retention.
2. The deployed “backend” has only static serving and `/health`; there is no
   durable tenant data, multi-user identity, account/site isolation, server
   audit store, backup, or subscription implementation for the target teams.
3. Phone/QR scanning is not implemented; the camera control only checks
   permission and falls back to typed/keyboard scanner input.
4. The $149/month Dock plan is advertised without checkout/entitlements or a
   matching claim test.

See [verification-2.md](./verification-2.md) for exact reproductions,
headers/privacy/rate-limit evidence, passed gates, and required next work.

How to reproduce the decisive data-loss defect: in a fresh browser import and
finalize PO-220 at `/start`, then import PO-221 at `/start`; returning to the
first PO route loads PO-221 because the browser DB stores only
`active-purchase-order`.

The Docker CLI was unavailable in the verification container. The exact
SHA-stamped web/API production build and clean-`PORT` binary startup were
verified locally; an image build/run remains for the deployment environment.
