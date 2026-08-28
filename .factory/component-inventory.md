# Intake Desk component inventory

This is the M1 component contract. Components use Svelte, semantic HTML, and
the tokens in `src/lib/design/tokens.css`. Domain rules remain in `lib/domain`;
storage/network work remains in adapters. Story/demo fixtures must cover every
state listed here without live services.

| # | Component | Purpose and contract | Required states | Keyboard / responsive behavior | First use |
| ---: | --- | --- | --- | --- | --- |
| 1 | `SiteHeader` | Skip link target, text wordmark, ≤4 route links, account/action slot, build-safe nav. | home/current route, demo, signed out/in, menu open | Native links; menu button announces state; at phone width links become a labelled sheet, not a horizontal scroll. | M1 |
| 2 | `DemoBanner` | Persistent “Demo — sample data, nothing is saved” notice with Reset and Start for real. | clean, changed, resetting, reset error | Banner is first after header; actions are buttons/links; stacks without hiding route H1. | M1 |
| 3 | `DockButton` | Primary, secondary, quiet, and danger actions with optional progress text. | default, hover, pressed, focus, disabled with reason, busy | Native button/link variants; ≥44 px; busy prevents repeat and announces; full-width only when phone task benefits. | M1 |
| 4 | `TextLink` | Consistent inline/standalone link with route or external affordance. | default, visited where useful, focus, external, download | Native anchor; underlined; external destination is stated in accessible name. | M1 |
| 5 | `Field` | Label, hint, input/select/textarea slot, units, validation and error summary anchor. | optional/required, valid, invalid, disabled, read-only, offline-saved | Label is always visible and bound; errors use `aria-describedby`; one column on phone. | M1 |
| 6 | `StatusStamp` | Word + compact geometry for due, partial, matched, short, damaged, pending, finalized, resolved. | all named statuses, quiet/strong, dark theme | Not interactive unless rendered inside a labelled control; never color-only; does not rotate text on phone. | M1 |
| 7 | `FilterRail` | PO/case filters with applied count and clear action. | no filters, applied, empty result, mobile open | Fieldset/legend or labelled controls; mobile opens in focus-managed sheet; results update is announced. | M1 |
| 8 | `PurchaseOrderRow` | Supplier, PO, due time, line count, receipt state and one clear route. | due, overdue, part received, closed, selected, loading | Whole-row destination is one anchor without nested controls; becomes labeled two-column pairs on phone. | M1 |
| 9 | `POManifest` | Header plus expected PO lines, source revision and supplier references. | loading, empty, loaded, stale, error | Semantic table on wide screens; definition-list rows on phone; no horizontal task scroll. | M1 |
| 10 | `QuantityStepper` | Decimal-string direct entry with minus/plus in declared unit and expected/difference context. | zero, matched, under, over, invalid, disabled, saving | Buttons and labelled input; arrow keys retain native input meaning; does not coerce blank/invalid silently. | M1 |
| 11 | `ScanCapture` | Keyboard-wedge burst/manual SKU capture; optional camera trigger later. | ready, listening, found, unknown, permission denied, unsupported | Ordinary labelled input; Enter resolves; no page-global keystroke theft; manual path stays visible at all widths. | M1 |
| 12 | `ConnectionStrip` | Exact persistence consequence: on device, syncing, synced, conflict, retry. | online, offline, queued, syncing, synced, conflict, failed | `role=status` only for changes; retry is a labelled button; stays in flow and wraps on phone. | M1 |
| 13 | `AttachmentTray` | Evidence thumbnails/files, captions, upload state, remove before finalize. | empty, local, uploading, uploaded, failed, rejected, storage full | File input label names types/size; remove names file; fixed thumbnail ratio; phone capture is optional. | M1 local fixture; M3 real upload |
| 14 | `DiscrepancyPanel` | Exact expected/received difference, kind, condition, note and resolution. | short, extra, damaged, wrong, partial, unresolved/resolved | Fieldset groups classifications; opening follows the mismatched line; stacks count before notes on phone. | M1 |
| 15 | `ReceiptTimeline` | Append-only event sequence with actor, time, payload summary, correction relation and hash status. | draft, finalized, corrected, resolved, verification failed | Ordered list; correction link targets earlier event; desktop rail becomes normal list on phone. | M1 |
| 16 | `ImportMapTable` | Source column → target mapping, sample and error, without hiding rejected rows. | unmapped, mapped, warning, invalid, duplicate, committing | Table/definition-list pairing; select labels include source column; error summary links to row. | M3 |
| 17 | `ExportMenu` | Plain receipt/discrepancy/complete export choices, schema/version and progress. | ready, preparing, download ready, expired, failed, read-only lapse | Button opens focus-managed menu; download is a real link; phone uses stacked list. | M1 CSV; M4 complete export |
| 18 | `ConfirmDialog` | Specific finalize, reset, revoke, delete or correction confirmation. | open, busy, failed, destructive/non-destructive | Native dialog where supported with tested fallback; trap/restore focus; Escape unless irreversible request is in flight. | M1 |
| 19 | `LiveNotice` | Polite immediate feedback, assertive only when action cannot continue. | info, success, warning, error, undo, timed/permanent | Focus does not jump for ordinary status; undo is reachable; never auto-dismiss errors or require hover. | M1 |
| 20 | `StatePanel` | Shared empty, loading, error, offline, no-permission, rate-limit and 404 composition. | each named state with next action and optional details | H2/H3 fits outline; loading reserves dimensions; action is first useful target; concise at 390 px. | M1 |

## Composition rules

- Routes own data and a single H1. Components never set `document.title` or
  create an H1 implicitly.
- A component may emit typed user intent. It does not import IndexedDB, fetch,
  MSAL, or the billing client.
- A route composes `StatePanel` rather than inventing one-off empty/error copy.
- `LiveNotice` is for action feedback; `ConnectionStrip` is the durable truth
  about sync; `StatusStamp` labels record state. Do not interchange them.
- Native HTML is preferred over headless libraries. Add a dependency only when
  its accessibility behavior is demonstrably better and the bundle stays in
  budget.
- Each interactive component gets a fixture for default, focus, disabled,
  loading, error, dark, reduced-motion, and 390 px states when applicable.

## M1 fixture names

Use stable fixtures so claim tests and screenshots do not drift:

- `po-nb-1047-open`
- `po-nb-1047-partial-count`
- `receipt-nb-1047-finalized`
- `receipt-nb-1047-corrected`
- `case-nb-1047-short-damaged`
- `connection-offline-queued`
- `storage-quota-error`
- `scanner-unknown-code`
