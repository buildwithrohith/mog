# Plan 017: Engine CPU scoping — kill the O(N²) materialize rebuild + small recalc wins

> **Executor instructions**: steps in order, each verified. STOP conditions
> per step. Update row 017 in `/Users/vish/Repos/analyst/mog/plans/README.md`
> when done (that one file is writable outside your worktree).
>
> **Drift check**: base `984efd41`;
> `git diff --stat 984efd41..HEAD -- compute/core/src/storage/engine/construction/deferred.rs compute/core/src/scheduler/` empty in your worktree.

## Status
P1 / M / Risk MED (browse-path hot code; strong existing tests) / Planned at `984efd41`.

## Why: measured evidence

Browsing all 28 sheets of the 1.4M-cell repro drove wasm memory to 4,135MB
(the old engine DIED at 4,086MB) and each tab click costs seconds. Cause:
`materialize_deferred_sheet_inner` (`construction/deferred.rs:358-591`)
rebuilds, on EVERY sheet-open, for ALL cumulative sheets: the workbook
snapshot (`parse_output_to_workbook_snapshot` at ~:461-466), grid/merge/
layout indexes (`all_sheets = 0..` at ~:534), and a full fresh
`CellMirror`+`ComputeCore` (~:558-577, wholesale `engine.stores.* = ...`
replacement). Opening sheet k reprocesses sheets 1..k-1: O(N²) CPU and the
wasm arena ratchet.

**The scoped pattern already exists IN THE SAME FILE and is test-covered**:
`hydrate_deferred_sheet_inner` (~:732-756, :819-847) builds indexes for ONE
sheet (`sheet_index..sheet_index+1`), then MERGES:
`engine.stores.grid_indexes.insert(sheet_id, ...)`,
`engine.mirror.replace_sheet(...)`,
`engine.mirror.replace_row_col_indexes_for_sheet(...)`. Port that pattern.

## Steps

### 1. Scope the materialize path (the main event)
Rework `materialize_deferred_sheet_inner` to: build grid/merge/layout for
ONLY the newly-materialized sheet (`sheet_index..sheet_index+1`) and
`.insert()`-merge into the existing maps; update the mirror via
`replace_sheet` + `replace_row_col_indexes_for_sheet` instead of a fresh
`CellMirror`; keep `ComputeCore` viewport-only state consistent — READ
`init_from_snapshot_viewport_only` (`scheduler/init.rs:324-362,420-467`)
first: it redoes formula-text normalization for all sheets; determine
whether a per-sheet variant exists or the compute store can be updated
incrementally like the hydrate path does (if the hydrate path solved this,
copy its approach; if it also rebuilds compute wholesale, scope ONLY what
it proves scopable and keep compute handling unchanged — do not invent new
incremental compute machinery in this plan).
KNOWN LIMIT (do NOT attempt here): `parse_output_to_workbook_snapshot`
still runs over the cumulative parse — that is a separate designed change
(README follow-up "incremental snapshot lowering"); your win is the index +
mirror rebuild elimination.
**Verify**: `cargo test -p compute-core --lib test_deferred_xlsx_import`
(all pass — bootstrap_rendering + existing_sheet_hydration cover switch
correctness incl. formats and revisits); full `cargo test -p compute-core
--lib` → 0 failures; `cargo check --workspace --locked`.

### 2. `ordered_sheets` Arc (every formula cell, small dirty levels)
`scheduler/level_eval.rs:31,56`: sequential path clones
`ordered_sheets_cache: Vec<SheetId>` per formula cell. Store as
`Arc<[SheetId]>`, clone the Arc. **Verify**: `cargo test -p compute-core --lib level_eval` + scheduler suite.

### 3. Recalc postprocess wins
(a) `storage/engine/recalc_postprocess.rs:58`: whole-`RecalcResult` clone
per mutation — establish who needs the copy vs a borrow/`mem::take`;
eliminate if the consumer allows (STOP the sub-step if ownership is
genuinely required, report why).
(b) `:124-127`: O(n²) `.any()` dedup over `validation_annotations` per
changed cell — use a `FxHashSet` of keys.
**Verify**: mutation + validation test modules
(`cargo test -p compute-core --lib -- --list | grep -i valid` to find them).

### 4. Enrichment memoization
`services/mutation_handlers/result_building/enrichment.rs:44-163`:
`get_hyperlink` + `get_resolved_cell_format` open a fresh Yrs read
transaction + 2 hex allocs PER CHANGED CELL. The sibling `comment_cache` in
the same file already memoizes per sheet — extend the same pattern to
hyperlinks and resolved formats (one transaction / one map per sheet per
enrichment pass). **Verify**: enrichment/mutation tests.

### 5. Mirror micro-fixes
`mirror/write/cells.rs:132-153`: `remove_cell` uses linear sheet scan —
use `cell_to_sheet.get()`. `mirror/types.rs:543-598` (esp. :574-579):
`rebuild_col_data` scans ALL `pos_to_id` keys for one column's rebuild —
scan only entries whose pos.col() matches via retain-style iteration (it
already does that check; the cost is iterating all — acceptable to leave if
a per-column index would be new state; in that case just fix remove_cell
and note the rest). **Verify**: mirror suite.

## Scope
**In**: files named above. **Out**: `parse_output_to_workbook_snapshot`
internals (designed follow-up); Yrs hydration internals (plan 015 owns);
`file-io/` (plan 016's lane).

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-j`, branch `opt/engine-cpu-scoping`,
commit per step, prefix `sapiex-patches:`, no push.

## Done criteria
- [ ] Materialize builds indexes/mirror for exactly one sheet per call
      (assert via a `#[cfg(test)]` counter or by test timing on a
      multi-sheet fixture — state which)
- [ ] Full compute-core lib → 0 failures; cargo check clean; no new fmt drift
- [ ] README row 017 updated per step

## STOP conditions
- Step 1: `replace_sheet`-based merging breaks any bootstrap_rendering
  correctness test in a way two attempts don't fix — report with the test
  name; do NOT weaken tests.
- Compute-store incremental handling requires new machinery — keep compute
  wholesale (documented), scope only indexes+mirror, report the split.
