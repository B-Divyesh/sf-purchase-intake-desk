# Factory handoff

Milestone M1 is built and deployed at
<https://purchase-intake-desk.sociobot.in>. The one-click isolated demo is at
<https://purchase-intake-desk.sociobot.in/?demo=1>.

See [handoff-m1.md](./handoff-m1.md) for shipped scope, verification evidence,
known gaps, the deliberate M1 boundary around auth/billing/server persistence,
and the M2 implementation checklist.

The required local gates are:

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build
cargo fmt --manifest-path api/Cargo.toml -- --check
cargo clippy --manifest-path api/Cargo.toml --all-targets -- -D warnings
```

Independent review and polish are still required before M2 begins.
