# Plan 008 design output: incremental per-sheet Yrs hydration

## Verdict

**FEASIBLE, with one required extension to the sketch:** Yrs hydration can be
made sheet-incremental, but calculation-bearing edits must hydrate the edited
sheet's **sheet dependency closure**, not only the edited sheet. Current
viewport-only initialization deliberately refuses to build a dependency graph
from a sparse workbook because formulas, names, and later-sheet cells may be
absent (`compute/core/src/scheduler/init.rs:351-405`). For example, editing
`Inputs!B2` while `DCF!G40 = Inputs!B2 * Revenue!G12` is still parse-only would
otherwise leave `DCF!G40` stale. A lightweight workbook-wide sheet-dependency
manifest must identify `Inputs -> DCF` and `Revenue -> DCF`; the engine then
materializes and Yrs-hydrates that closure before admitting the edit.

This design was checked on branch `sapiex-patches` at exact commit
`80f56679676031b20f2010f17c844e5efd5e9d9e` (`80f56679`). The current source
does not trigger the STOP condition: it already has selected-sheet parsing,
stable prior-allocation reuse, an in-place style-only transaction, bootstrap
undo origin, and per-sheet hydration internals. The implementation is not a
small extraction, however: it must add an existing-sheet hydration primitive,
one allocator authority, dependency-closure admission, and complete sync/export
guards.

## 1. Chosen architecture

Use **per-sheet Yrs subtrees hydrated in independent transactions**. Retain the
deferred import record while any imported sheet is incomplete, and replace its
binary meaning with explicit state:

```rust
struct IncrementalHydrationState {
    raw_xlsx_bytes: Vec<u8>,
    parse_output: ParseOutput,             // cumulative selected sheets
    workbook_snap: WorkbookSnapshot,       // cumulative selected sheets
    allocations: Vec<SheetIdAllocation>,
    mirror_materialized_sheets: HashSet<SheetId>,
    yrs_hydrated_sheets: HashSet<SheetId>,
    expected_imported_sheets: HashSet<SheetId>,
    sheet_dependency_manifest: SheetDependencyManifest,
}
```

The sets are monotonic and obey
`yrs_hydrated_sheets ⊆ mirror_materialized_sheets ⊆ expected_imported_sheets`.
The landing sheet starts in both materialized sets; other sheet maps exist in
Yrs for tab metadata/styles, but they do **not** count as Yrs-hydrated until
their cells, axes, grid index, ranges, and sheet domains are resident. The
global completion predicate is exact set equality, not a nonempty sheet order.

`ensure_yrs_sheet_hydrated(sheet_id)` performs this sequence:

1. If parse-only, reuse `parse_selected_sheets` and the cumulative sparse
   rebuild already used by `materialize_deferred_sheet_inner`
   (`compute/core/src/storage/engine/construction/deferred.rs:342-450`).
2. Allocate any new row/column/cell identities from the engine's shared
   monotonic grid allocator, while reusing prior positional IDs. Current reuse
   consumes allocator slots and preserves earlier IDs
   (`compute/core/src/storage/infra/hydration/sheet/allocation.rs:42-118`); the engine allocator exposes an
   atomic high-water mark and `ensure_past`
   (`compute/core/crates/types/cell-types/src/id_alloc.rs:197-240`). This is essential after edits: if an
   edit handed out ID 900, later hydration must start past 900, not merely past
   the import allocations.
3. Build the target sheet's range/style plan, then open one transaction with
   `ORIGIN_BOOTSTRAP`. Under `engine.mutation.suppress_guard()`, replace the
   target's existing metadata/style-only subtree with its canonical full
   subtree. The new hydration primitive must neither append to `sheetOrder` nor
   rewrite workbook-global roots. The existing helper currently always appends
   and inserts a new sheet map
   (`compute/core/src/storage/infra/hydration/sheet/mod.rs:517-542`), so it cannot
   be called unchanged.
4. Commit the transaction before adding the sheet to
   `yrs_hydrated_sheets`. On any error, the set and live subtree remain at the
   prior state; retry is safe.
5. Rebuild/refresh only the target sheet's grid, merge, layout, mirror, formula,
   and phantom-cell indexes. Do not reconstruct all cumulative sheets as the
   preview path currently does
   (`compute/core/src/storage/engine/construction/deferred.rs:517-563`).

### Calculation-bearing edit admission

At initial import, stream worksheet formula references into a compact
`SheetDependencyManifest` keyed by workbook sheet identity. It stores
sheet-level precedent/dependent edges and named-range sheet dependencies, not
all `CellData`. Before a value/formula edit, compute the transitive sheet
closure needed to evaluate every affected formula, materialize/hydrate those
sheets one at a time, and register their formulas into the scheduler. A newly
typed formula is parsed during preflight so its prospective references also
expand the closure before the formula is written. For a
formula-free workbook, editing one new sheet hydrates exactly one new sheet.
For a connected financial model, an edit can legitimately hydrate several
tabs or the whole workbook; correctness wins over pretending `K = 1`.

Non-calculation metadata edits that are provably sheet-local (for example a tab
color) need only the target sheet. Structural edits remain all-sheet
operations in the first implementation because inserting a row in `Inputs`
can rewrite formulas on unhydrated sheets; they call sequential completion
before mutation. This preserves the current structure-change guarantee, which
stages full hydration before applying the change
(`compute/core/src/storage/engine/structural/structure_change.rs:10-60`), while eliminating its single
giant transaction.

## 2. State machine and operation contract

| State | Canonical contents | Legal operations | Transition trigger |
|---|---|---|---|
| `ParseOnly` | Workbook inventory and sheet metadata only; no sheet cells in mirror or full Yrs subtree | List/switch tabs, read workbook/sheet metadata, request viewport materialization | View -> `MirrorMaterialized`; edit -> hydrate dependency closure through `YrsHydrated` |
| `MirrorMaterialized` | Selected-sheet parse/snapshot, mirror/index data, and preview styles; cached values are readable; full Yrs cells/grid index absent | View/query cells and styles; preview-only UI overlays; request edit | Edit -> `YrsHydrated`; leaving the sheet does not demote it |
| `YrsHydrated` | Full canonical sheet subtree plus matching mirror/index state | Sheet-local edits; formula edits only after the dependency closure is also hydrated; undo/redo for admitted user edits | First edit, explicit warm-up, dependency closure, or sequential completion |

Concrete transitions:

- **View `Retention`**: `ParseOnly -> MirrorMaterialized`. This is today's
  working path: `Retention!A1` becomes queryable while `Usage` remains without
  a cell ID
  (`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/bootstrap_rendering.rs:292-376`). It does not emit provider data.
- **Edit `Retention!A1`**: build the dependency closure, then transition every
  member through `MirrorMaterialized -> YrsHydrated`; only after hydration
  commits does the normal user transaction run. Hydration never shares a
  transaction or undo item with the edit.
- **Format/clear a range**: same target-sheet preflight. Existing filters
  explicitly reject sparse graph state before mutation
  (`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/bootstrap_rendering.rs:628-668`); the new preflight must hydrate first and
  retain the no-partial-mutation property.
- **Insert/delete rows or columns, rename/delete/reorder a sheet, or mutate a
  workbook-scoped name/table**: sequentially hydrate all missing sheets, then
  execute the existing operation. A later design may journal structure changes
  against raw XLSX, but this design does not create that second source of truth.
- **Export/save**: all sheets must be `YrsHydrated`; see section 3.
- **Sync/provider attach/apply**: refused until all sheets are `YrsHydrated`;
  see blocker 4 below.

There is no eviction transition in this design. Once a sheet reaches
`MirrorMaterialized` or `YrsHydrated`, it stays there for the engine lifetime.
Eviction would need coordinated kernel materialization tracking and is a
separate architecture problem.

### Blocker evidence and resolution

1. **Single transaction / binary guard — verified.** Full completion creates a
   fresh `YrsStorage`, hydrates every sheet in one transaction, builds all
   stores, then swaps the engine
   (`compute/core/src/storage/engine/construction/deferred.rs:766-887,
   969-1002`). Export currently rejects solely on
   `deferred_hydration.is_some()`
   (`compute/core/src/storage/engine/export.rs:190-217`), and the empty-doc
   commit guard is at
   `compute/core/src/storage/engine/construction/deferred.rs:844-850`. Replace the binary
   predicate with set equality and retain the empty/nonempty consistency check
   as a second invariant.
2. **Allocator coupling — verified.** Current selected-sheet materialization
   seeds above all retained allocations and recomputes with prior-allocation
   reuse (`compute/core/src/storage/engine/construction/deferred.rs:405-421`);
   `seed_after_allocations` scans sheet/row/col/cell/identity IDs
   (`compute/core/src/storage/engine/construction/deferred.rs:895-920`). Share
   the live grid allocator with later hydration so IDs allocated by intervening
   edits also advance the authority.
3. **Observer/undo suppression — verified.** Style hydration uses
   `suppress_guard`, one transaction, and clears its update payload
   (`compute/core/src/storage/engine/construction/deferred.rs:480-515`). Undo tracks only `ORIGIN_USER_EDIT`
   and `ORIGIN_STRUCTURAL`; `ORIGIN_BOOTSTRAP` is intentionally excluded
   (`compute/core/crates/compute-document/src/undo/origin.rs:4-40`). Each per-sheet transaction uses
   both mechanisms.
4. **Collaboration/sync — current guard is insufficient.** `encode_diff` only
   rejects the narrower empty-sheet-order/nonempty-mirror state
   (`compute/core/src/storage/engine/sync_bridge.rs:107-120`), and a regression currently asserts that a
   critical partial workbook can encode and replay
   (`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/provider_replay.rs:195-251`). The minimal new contract refuses
   `apply_sync_update`, `encode_diff`, and `drain_pending_updates` while the set
   is incomplete. `encode_state_vector` may remain a local diagnostic, but the
   kernel must not attach a provider in this state.
5. **`idToPos` — verified and decided in section 6.** It has direct read
   consumers, but the converged snapshot path already treats `posToId` as the
   winner and calls `idToPos` an inverse that can retain losing CellIds
   (`compute/core/src/storage/engine/construction/snapshots.rs:218-280`).

### Per-transaction update-buffer semantics

`UpdateSource` is attached by the subscription callback, not inferred from
transaction origin (`compute/core/src/storage/engine/update_buffer.rs:48-105`).
The current deferred path installs `ImportBootstrap` after initial import and
`FullHydration` after the full swap
(`compute/core/src/storage/engine/construction/deferred.rs:287-292,975-981`).
Therefore incremental hydration must deliberately swap the subscription source;
semantic suppression alone cannot classify provider bytes.

The engine dispatch actor serializes these steps:

1. Install/tag the observer as `FullHydration` for the bootstrap sheet
   transaction.
2. Run the `ORIGIN_BOOTSTRAP` transaction under semantic suppression.
3. Clear exactly that transaction's update bytes.
4. Reinstall the ordinary `UserMutation` observer **before** admitting the user
   edit.

This avoids clearing a real edit after a later hydration. Import/hydration is
base state delivered by a full-state diff, never live provider fan-out; the
existing regression establishes that bootstrap and completion queues are empty
while the first post-import edit does enqueue an update
(`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/provider_replay.rs:5-52`). Add a regression that hydrates, edits, hydrates a
second sheet, and proves the edit remains in document state while no hydration
update reaches the provider queue.

## 3. Export contract

Choose **explicit sequential hydrate-before-export**, exposed to the caller as
progress, rather than mutating inside the current read-only export method.

`export_to_parse_output` currently takes `&self` and runs a require guard before
reading Yrs (`compute/core/src/storage/engine/export.rs:190-217`). Keep that separation:

- If `missing = expected_imported_sheets - yrs_hydrated_sheets` is nonempty,
  export returns a structured actionable error containing `missing_count`,
  missing sheet IDs/names (bounded preview), and the required action
  `complete_deferred_hydration`.
- The kernel save/export flow invokes redesigned
  `complete_deferred_hydration`, which hydrates each missing sheet in workbook
  order, emits `{completedSheets,totalSheets,currentSheet}` progress, performs
  the final graph/recalc policy, clears bootstrap update bytes, then retries
  export.
- A failure on sheet 17 leaves sheets 1-16 durably resident in the engine and
  leaves the global guard closed. Retrying resumes at sheet 17.
- The final require guard checks exact set equality, nonempty Yrs order for a
  nonempty mirror, and equality between expected and Yrs sheet identities.

Example: with 28 sheets and only `Inputs`, `Revenue`, and `DCF` hydrated,
export reports 25 missing sheets; it never serializes a three-sheet workbook as
if it were complete. This preserves plan 003's invariant and strengthens the
existing partial-export test, which today only checks the binary deferred flag
(`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/partial_export.rs:5-38`).

## 4. Memory math

The current measured native fixed sizes are:

| Type | Bytes per cell | Evidence |
|---|---:|---|
| `CellData` | 600 | `compute/core/tests/type_size_report.rs:14-46`; focused test printed 600 |
| `CellEntry` | 64 | same focused test; the value is inline at `compute/core/src/mirror/types.rs:113-123` |
| `CellValue` | 56 | size report; already contained in the two rows above, so do not add it again |
| `CellId` | 16 | size report |
| Yrs document estimate | 1,200-1,600 | round-1 measured estimate supplied by plan 008 |

For a working set of `W` hydrated/materialized cells, the fixed-plus-Yrs
planning estimate is therefore:

```text
peak_working_set ~= W * (600 + 64 + 1,200..1,600)
                 ~= W * 1,864..2,264 bytes
```

This deliberately excludes dynamic strings, hash-table slack, range payloads,
formula AST/graph storage, raw XLSX bytes, styles, row/column IDs, and wasm
allocator fragmentation; it is a floor/range for comparison, not a 4 GiB
safety proof.

The 1,407,921-cell, 28-sheet repro averages 50,282.9 cells per sheet. If sheets
were equal-sized and `K` means edited **plus dependency-closure** sheets beyond
the landing sheet:

| `K` | Resident sheets | Yrs only | Fixed parse + mirror + Yrs estimate |
|---:|---:|---:|---:|
| 0 | 1 | 60.3-80.5 MB | 93.7-113.8 MB |
| 1 | 2 | 120.7-160.9 MB | 187.5-227.7 MB |
| 2 | 3 | 181.0-241.4 MB | 281.2-341.5 MB |
| 3 | 4 | 241.4-321.8 MB | 374.9-455.4 MB |

At all 28 sheets, Yrs alone is about 1,689.5-2,252.7 MB; adding the 600-byte
parse floor (844.8 MB) and 64-byte mirror floor (90.1 MB) gives
2,624.4-3,187.5 MB before all excluded overhead. That explains why the current
all-at-once staging peak can cross wasm32's 4,077 MB ceiling even though the
final nominal components look smaller. The incremental design removes the
simultaneous fresh-full-doc plus full parse/snapshot/mirror staging peak. Real
acceptance must use actual per-sheet cell counts; one 600,000-cell landing sheet
costs approximately 1.12-1.36 GB by the same formula regardless of the
28-sheet average.

## 5. Dispatch-ready implementation plan

| Step | Size | Change and files | Regression / acceptance |
|---|---|---|---|
| 1. Make hydration state explicit | S | Add the two sets, expected inventory, predicates, and structured incomplete error in `compute/core/src/storage/engine/construction/types.rs`, `engine/mod.rs`, and `engine/construction/deferred.rs`. | Extend `test_deferred_xlsx_import/guards.rs`: landing is hydrated, other sheets are not; equality alone clears the global guard; an empty/inconsistent set never does. |
| 2. Unify allocation authority | M | Add a hydration allocator adapter over the shared `cell_types::IdAllocator`; replace later-sheet `DefaultIdAllocator` reseeding in `construction/deferred.rs`; touch `storage/infra/hydration/mod.rs` and `sheet/allocation.rs`. | Extend `identity_allocation.rs`: hydrate A, edit A to allocate IDs, hydrate earlier/later B, assert every sheet/row/col/cell ID is unique and A's IDs remain stable. |
| 3. Add existing-sheet hydration | M | Extract an in-place target-sheet variant from `storage/infra/hydration/import.rs` and `sheet/mod.rs`; it replaces one subtree, writes ranges/styles/domains, does not append order, and does not rewrite workbook roots. Wire it from `construction/deferred.rs`. | New focused cases beside `bootstrap_rendering.rs`: one transaction per sheet; stable sheet order; styles/ranges/comments/tables preserved; edit A then materialize B preserves A in Yrs and mirror; failure does not mark the sheet hydrated; repeat call is idempotent. |
| 4. Add the dependency manifest and closure admission **(Riskiest)** | L | Stream sheet formula/name references in `file-io/xlsx/parser/src/pipeline/full_parse/` and expose them through `file-io/xlsx-api`; add closure state in `construction/types.rs`; add incremental formula registration/readiness in `compute/core/src/scheduler/init.rs` and engine preflight. | Three-sheet case: edit an input, hydrate its downstream/precedent closure, recalc cross-sheet formula correctly, leave unrelated sheet parse-only. Strongly connected case hydrates the full component. Named-range and external-reference cases fail closed rather than returning stale values. |
| 5. Route mutation classes | M-L | Add sheet-set preflight to central mutation dispatch and direct sheet mutation front doors in `engine/mutation_dispatch.rs`, `cell_bridge.rs`, `delegations/`, `formatting/`, `features/`, and `structural/structure_change.rs`. Structural/workbook mutations call sequential full completion. | Table-driven tests cover cell, batch, format, filter, comment/object, structural, and workbook-scoped operations; no mutation lands before required hydration; rejected operations leave state byte-identical. |
| 6. Make bootstrap transactions invisible to undo/providers | M | Use `ORIGIN_BOOTSTRAP`, `suppress_guard`, per-hydration observer source, exact buffer clear, then reinstall `UserMutation` in `construction/deferred.rs` and `update_buffer.rs`. | Extend `provider_replay.rs`: hydration creates no undo item/update; user edit does; a later sheet hydration neither clears nor reclassifies that edit. |
| 7. Strengthen export and sync contracts | M | Redesign `complete_deferred_hydration` as resumable sheet iteration with progress in `bridge_imports.rs`; use set equality in `export.rs`; refuse partial `apply_sync_update`, `encode_diff`, and `drain_pending_updates` in `sync_bridge.rs`; update kernel save/provider attach handling. | Replace the current critical-partial replay success test with refusal; test 28-sheet progress/resume, failure at N, retry, full export parity, and full provider replay. |
| 8. Prove the hard memory case | M | Add counters/sampling around each target transaction and a noncommitted repro harness using the existing plan-007 wasm telemetry. | On the 1,407,921-cell repro: landing open succeeds; edit at least two nonlanding sheets; measured peak tracks their real closure; sequential completion/export either stays below wasm ceiling and round-trips 28 sheets or reports the exact sheet/peak where the final resident state itself is too large. |
| 9. Remove `idToPos` in schema v20 | L, separate cutover after steps 1-8 | See section 6. Touch `compute-document/schema.rs`, hydration/grid writers, snapshot/undo/sync repair, comments, sheet copy, canonical tests, and provider persistence migration. | v19 -> compact v20 migration; v20 direct import/edit/undo/structural/sync/copy/comment tests; older binary rejects v20; no writer recreates `idToPos`. |

Step 4 is the single riskiest step. Without it, independent Yrs transactions
solve the memory spike but not correct automatic calculation: the scheduler's
current sparse guard would still reject edits, or removing the guard would
silently miss cross-sheet dependencies.

## 6. `idToPos` verdict: REMOVE via a versioned, compacting cutover

**Verdict: remove `gridIndex/idToPos`; keep `posToId` as the sole persisted
identity ownership map. Do not replace the Y.Map keys with packed `u64`: Y.Map
keys are strings, so packing only moves encoding complexity without removing
the second CRDT entry.**

The source already establishes the right authority: snapshot rebuild iterates
`posToId` because it is the CRDT winner, while `idToPos` may contain losing IDs
after concurrent writes
(`compute/core/src/storage/engine/construction/snapshots.rs:218-280`). Named-range
normalization prefers `posToId` and uses `idToPos` only when no position entry
was inserted
(`compute/core/src/storage/engine/construction/named_ranges.rs:167-215`). There are no kernel or
app literal consumers in the current tree; the load-bearing Rust consumers are:

- undo/observer recovery's `read_cell_position_from_yrs`
  (`compute/core/src/storage/cells/values/storage_methods.rs:88-150`);
- position deletion's reverse lookup
  (`compute/core/src/storage/cells/values.rs:108-139`);
- sync orphan repair
  (`compute/core/src/storage/engine/sync_pipeline/repair.rs:15-128,138-215`);
- comment identity validation
  (`compute/core/src/storage/sheet/comments/cleanup.rs:12-33`);
- sheet-copy ID remapping (`compute/core/src/storage/sheet/crud.rs:174-216`);
- schema construction/assertions (`compute/core/crates/compute-document/src/schema.rs:20-33,
  625-636`) and canonical test normalization.

Replace those reads with a transient reverse index built once from `posToId`
when rebuilding a sheet. Deletion paths already know or can obtain the current
position from the live `GridIndex`; pass the position key into the delete helper
instead of persisting a second map. Comment checks become membership in the
transient cell-ID set. Sync repair iterates only the authoritative winner map.
Sheet copy rebuilds its reverse relationships from the remapped `posToId` map.

Migration must be a schema cutover, not indefinite dual-write:

1. Bump schema 19 -> 20 and make v20 writers create only `posToId`.
2. A v20 binary loads v19 semantically, rebuilds a fresh v20 `Doc` from the
   authoritative `posToId` state, and persists/attaches providers from that
   compact full state. Simply deleting the Y.Map in place may retain CRDT
   history/tombstones and does not prove the memory win.
3. Coordinate provider baseline replacement because state vectors from the old
   document history are not a v20 migration protocol. Older binaries already
   have a max-supported schema check and must refuse v20 rather than recreate
   the inverse map.
4. Land all readers/writers/tests in one cutover; no legacy writer remains.

This removes an estimated 40-50% of per-cell Yrs cost, but it should follow the
incremental hydration work rather than share its first production cut: schema
migration and hydration state are independently high-risk and need separable
rollback evidence.

## 7. Rejected alternatives

### Background full hydration in chunks only

Rejected as the architecture, retained as an optional warm-up policy. Chunking
the current all-workbook operation without sheet state still eventually builds
the entire Yrs document, does not tell edit/export/sync which sheets are safe,
and cannot resume precisely after a sheet failure. For example, if the user
edits sheet 3 while a blind background pass is on sheet 17, the engine still
needs an explicit sheet guard to know whether sheet 3 is durable. The chosen
architecture can run the same per-sheet transition in workbook order during
idle time, but foreground edits may prioritize their dependency closure.

### Server-side-only editing for oversized workbooks

Rejected. It changes the product/runtime contract, introduces network latency
and offline failure for every edit, and leaves browser provider/undo semantics
split across two engines. For example, a user typing in `Inputs!B2` would depend
on a remote round-trip merely because the workbook crossed a size threshold.
Server-side export can be a separately designed escape hatch if the **final
resident** v20 document itself cannot fit wasm32, but it is not the editing
architecture and must not hide a partial local document.

### Hydrate only the directly edited sheet and ignore the graph guard

Rejected for correctness. `ensure_graph_built` rejects sparse viewport state
because cross-sheet formulas and names can be absent
(`compute/core/src/scheduler/init.rs:364-405`). Deleting that check would make a green local
edit test stand in for a stale workbook. The dependency manifest/closure is the
minimum honest relaxation.
