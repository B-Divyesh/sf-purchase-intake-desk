# Repair 3 verification record

Date: 2026-09-05

Implementation tested and deployed:
`5e3cef391ac4811d9c6439ea781f875adfda9aac`.

## Earlier finding disposition

| Earlier finding | Disposition | Outcome evidence |
| --- | --- | --- |
| Global PO and receipt IDs overwrite another tenant | Fixed | `@claim:server-tenant-isolation` creates identical IDs in two tenants, reloads both, and proves each payload remains separate. |
| Client cannot reload server records; membership opens the wrong tenant | Fixed | `@claim:server-record-reload` assigns a second user, reopens SQLite, and reloads the assigned site, PO, finalized receipt, events, and evidence metadata. |
| Server audit timestamps are fixed | Fixed | `@claim:server-audit-retention` bounds the timestamp to request time, rejects the old constant, reopens the database, and checks the retained value. |
| Corrections are local; failures are ignored; evidence cannot be retrieved; WAL copy is unsafe | Fixed | The server appends correction events, every client write checks status, authenticated evidence download returns original bytes, and SQLite backup API output reopens successfully. |
| Entitlements are recorded but not enforced | Fixed | `@claim:entitlement-read-only` proves active writes work, expired/revoked writes return 402, and reads still work. |
| Authenticated responses can enter the service-worker cache | Fixed | `@claim:authenticated-cache-bypass` poisons a same-URL cache entry, makes an authorized fetch, and proves the cached tenant value is never returned. |
| Public server promises lack claim checks; checkout claim checks only a link | Fixed | Claims now cover isolation, reload, audit retention, entitlement, and cache bypass. `@claim:checkout-unavailable` proves the price is labeled planned and no checkout link is offered. |
| 401 omits Bearer challenge; JWKS refreshes each request; no session restore/sign-out | Fixed | API integration checks `WWW-Authenticate`; verifier caches discovery/JWKS for one hour; client restores session on `/start` and `/app` and exposes sign-out. Hosted sign-in remains an operator check. |
| Zero quantities import | Fixed | CSV validation rejects zero and negative order quantities; unit and browser invalid-path coverage pass. |
| Service-worker navigation masks live 404 | Fixed | Controlled online browser navigation to `/cold-live-missing-page` returns HTTP 404 and the designed H1. |
| Duplicate description and canonical metadata | Fixed | Route checks assert exactly one description and canonical element. |
| Landing preview creates a nested complementary landmark | Fixed | Preview is no longer an `aside`; axe reports no violations on all tested routes and both viewports. |
| Price contradicts the researched brief | Fixed | Copy consistently describes $149/site/month as planned and checkout as unavailable. |

## Boundaries proved

- The demo creates only `intake-desk:demo:v1`, sends no `/api` request, and
  reset restores NB-1047. It does not read or write the real workspace.
- Production tables use `(tenant_id, site_id, id)` ownership keys. Route access
  resolves membership before every read or write.
- The service worker bypasses API and authorized requests and never uses
  `ignoreVary`.
- The live container uses one replica and its own durable product share at
  `/data`; no other product resource was accessed.

## Live check summary

The cold desktop and 390 px phone records are in
`evidence-repair-3-live/cold-browser.json`. Both first screens show:

- Job: “Check deliveries against the purchase order.”
- Audience: small receiving teams needing a record before the supplier leaves.
- First action: “Try it with sample data.”

Both contexts opened realistic sample data, showed the persistent demo label,
reset a changed belt count to 46, used no real workspace, and reported no
unexpected console error. A deliberate missing URL returned HTTP 404; that is
expected behavior, not a failed page.

The live Lighthouse report scored 98/100/100/100. The URL verifier recorded a
656 ms desktop load in its environment with one H1, a main landmark, no missing
alt text, no unlabeled buttons, and no console error.
