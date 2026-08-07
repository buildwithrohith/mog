# Plan 008: DESIGN — incremental per-sheet Yrs hydration (ARCH-01)

> **This is a design task, not an implementation task.** The deliverable is a
> design document written to `plans/008-DESIGN-OUTPUT.md`, plus a verdict on
> feasibility. Do NOT modify source code. You may add throwaway probe tests
> under `compute/core/tests/` ONLY if needed to verify a claim, and must
> delete them before finishing.
>
> **Drift check (run first)**: `git rev-parse --short HEAD` must be
> `80f56679` or a descendant on `sapiex-patches`; note the exact hash in
> your output doc.

## Status

- **Priority**: P1 (the architectural end-state; everything else is diet)
- **Effort**: L (design), implementation TBD by this design
- **Risk of design work**: none (no code changes)
- **Depends on**: plans 001-007 landed (they are; base `80f56679`)
- **Planned at**: commit `80f56679`, 2026-08-06

## The problem this design must solve

Full Yrs hydration (`stage_deferred_hydration` + `commit_deferred_hydration`
in `compute/core/src/storage/engine/construction/deferred.rs`) writes the
ENTIRE workbook into the Yrs CRDT document in one pass inside one
transaction. Measured costs at the 1,407,921-cell / 28-sheet repro:

- Yrs per-cell cost ~1.2-1.6KB (dominant: the `posToId` + `idToPos` double
  index, estimated 40-50% of that per-cell cost).
- The wasm32 build OOMed at 4,077MB inside hashbrown `reserve_rehash`
  during this pass (pre-fork bug; the deleted skip-branch was the old
  workaround; plan 003 removed it so oversized full hydration now fails
  VISIBLY instead of producing an empty document).
- Read-only viewing no longer needs full hydration at all (per-sheet
  materialization, plans 001/004), but EDITING an oversized workbook still
  requires full hydration and therefore still cannot exceed the ~4GiB
  ceiling on wasm32.

The architectural fix: hydrate the Yrs document PER SHEET, on demand —
landing sheet at import, other sheets when first edited (or in background),
so peak memory scales with the visited/edited working set, not the workbook.

## Known blockers (inventoried during round-1 audit; verify each in source)

1. **Single-transaction assumption**: `stage_deferred_hydration` builds
   `new_storage` (a fresh `YrsStorage`) fully, then `commit_deferred_hydration`
   swaps it in and clears the export guard (`engine.deferred_hydration = None`)
   — export correctness currently equals "all sheets present in Yrs".
   Plan 003 added: staging refuses to commit when the parse output has sheets
   but `new_storage.sheet_order()` is empty, and export/sync refuse an empty
   sheet order with a non-empty mirror (`export.rs` require-guard,
   `sync_bridge.rs:encode_diff`). A per-sheet design must replace the binary
   "deferred_hydration.is_some()" guard with a per-sheet materialization set,
   and export must refuse unless EVERY sheet is Yrs-resident (or must
   hydrate-on-export the missing ones).
2. **ID allocator coupling**: cell/row/col ids are allocated per sheet during
   hydration (`hydration/sheet/allocation.rs`,
   `allocate_sheet_ids_with_previous_allocation`); the allocator seed is
   derived from high-water marks (`seed_after_allocations`,
   `snapshot_id_high_water_mark`). Round-1 made id vectors `Arc`-shared.
   Incremental hydration must keep later-sheet allocations consistent with
   ids already handed out (the deferred materialization path already does
   exactly this for the MIRROR side — study it as the template).
3. **Observer/undo suppression**: hydration writes must not enter undo
   history or be tagged as user mutations. The suppression mechanisms exist
   and are in use TODAY: `engine.mutation.suppress_guard()` (used by plan
   004's style-only hydration in `materialize_deferred_sheet_inner`) and
   `UpdateSource::{ImportBootstrap,FullHydration}` tagging on
   `install_observer` (plan 007). Per-sheet hydration transactions can reuse
   both; the design must specify the update-buffer semantics per transaction
   (see `engine.update_buffer.clear()` precedents at the two existing sites).
4. **Collaboration/sync semantics**: a partially-hydrated Yrs doc syncing to
   a provider would replicate a partial workbook. `encode_diff` now refuses
   inconsistent state (plan 003), but the design must decide: is sync simply
   refused until fully hydrated (status quo, acceptable), or does the sync
   layer gain awareness? Recommend the minimal contract and state it.
5. **`idToPos` cost**: since hydration cost is dominated by the double index,
   evaluate ONE bundled schema question: can `idToPos` be removed or replaced
   (e.g., position-keyed only, deriving id→pos lazily; or packed u64 keys
   instead of hex strings)? This changes the Yrs document schema, which
   affects persistence/compat — enumerate what reads `idToPos`
   (`compute-document` crate, sync consumers) and give a
   keep/replace/remove verdict with migration notes. If the answer is "keep",
   say why in one paragraph.

## What the design document must contain

1. **Chosen architecture** (one): per-sheet Yrs subtrees hydrated in
   independent transactions, with a `materialized_yrs_sheets: HashSet<SheetId>`
   (or equivalent) guard replacing the binary deferred flag. Or argue for a
   different shape if the source contradicts this sketch.
2. **State machine**: the exact states a sheet can be in
   (parse-only → mirror-materialized → yrs-hydrated), which operations are
   legal in each, and which transitions each user action triggers (view,
   edit, structure change, export, sync).
3. **Export contract**: how export works when some sheets are not yet
   Yrs-resident (hydrate-on-export with progress? refuse with actionable
   error?). Must preserve plan 003's "never silently empty" invariant.
4. **Memory math**: expected peak for the 1.4M-cell repro under the design
   (landing sheet + K edited sheets), using the measured per-cell numbers
   from plan 007's `type_size_report` and the audit's Yrs per-cell estimate.
5. **Implementation plan skeleton**: ordered steps sized S/M/L, each with the
   files it touches and its regression tests, so implementation lanes can be
   dispatched directly from it. Flag the single riskiest step.
6. **The `idToPos` verdict** (blocker 5).
7. **Rejected alternatives**: at least background-full-hydration-in-chunks
   (why or why not) and server-side-only editing for oversized books.

## Commands you will need

| Purpose | Command |
|---|---|
| Read the two existing per-sheet precedents | `materialize_deferred_sheet_inner` and plan 004's style hydration block in `construction/deferred.rs`; `hydrate_sheet_styles_only` in `storage/infra/hydration/styles.rs` |
| Full hydration path | `stage_deferred_hydration` / `commit_deferred_hydration` in the same file |
| Yrs schema | `compute-document` crate (`schema` module: KEY_* constants) |
| Existing tests to study | `compute/core/src/storage/engine/tests/test_deferred_xlsx_import/` (guards.rs, calc_completion.rs, bootstrap_rendering.rs) |

## Scope

**In scope**: reading everything; writing `plans/008-DESIGN-OUTPUT.md`.
**Out of scope**: ANY source modification; ANY commit to the repo.

## Done criteria

- [ ] `plans/008-DESIGN-OUTPUT.md` exists with all 7 sections above
- [ ] Every blocker (1-5) addressed with file:line evidence from CURRENT source
- [ ] Implementation skeleton is dispatch-ready (steps, files, tests, risk)
- [ ] README row 008 updated (DONE = design delivered, with a one-line verdict)

## STOP conditions

- The source contradicts the blocker inventory in a way that invalidates the
  per-sheet approach entirely — write up WHY in the output doc and mark the
  row BLOCKED with the reason (that is still a successful design outcome).
