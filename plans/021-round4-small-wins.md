# Plan 021: Round-4 small wins — padding bound, silent-drop diagnostics, batched #REF! recompute

> Steps independent; commit each; if one STOPs, continue to the next. Update
> row 021 in plans/README.md when done.
> **Drift check**: base `02403d9b`; your files:
> `compute/core/src/mirror/{types.rs,structure.rs,write/projection_materialization.rs}`,
> `file-io/xlsx/parser/src/write/from_parse_output/sheet_cells.rs`,
> `kernel/src/bridges/compute/compute-bridge.ts` (+ a bridge method in
> `compute/core/src/storage/engine/queries.rs`). Confirm
> `git diff --stat 02403d9b..HEAD -- <those>` empty in YOUR worktree.

## Status
P2 bundle / S+S+M / Risk LOW-MED / Planned at `02403d9b`.

## Step 1 (M): Bound dense column padding to per-column extent
Evidence: `mirror/types.rs:543-598` `rebuild_col_data` computes per-column
`max_row` then floors at sheet rows (`let size = max_row.max(self.rows as usize)`);
`structure.rs:577-596` resizes EVERY col_data vec to `s.rows`;
`projection_materialization.rs:30-41` floors at `sheet_mirror.rows`. One
far-down row anywhere makes EVERY range-backed column pay
`sheet.rows × 24B` of Null padding (~12MB per column at 500K rows; ~240MB in
a plausible 20-column case). Readers already tolerate short vectors
(`dense.rs:87-88` clamps `(rows as usize).min(col_slice.len())`).
Change: size each column's vec to ITS OWN max populated row (+ the range/spill
extent that column actually has), not `sheet.rows`. AUDIT every direct
`col_vec[row]` indexer first (grep `col_data`) — any site assuming full
length gets a bounds-checked accessor. Add a test: sheet with rows=500_000
(one cell at row 499_999 in col A), a range-backed col B with 10 top rows →
assert col B's vec length is B's extent, not 500_000, and reads at high rows
return Null.
**Verify**: `cargo test -p compute-core --lib` 0 failures (mirror + range
cache suites are the net).

## Step 2 (S): Diagnose the silent AuthoredStyleRun drop on export
Evidence: `write/from_parse_output/sheet_cells.rs:98-111` — when
`emitted_cell_xf_id(run.style_id)` is None (and style_id != 0) the whole
authored run vanishes with no signal. Add a counted diagnostic (the write
side's report/summary struct if one exists — grep how the writer surfaces
warnings; else `tracing::warn!` once per sheet with the drop count) and a
test constructing a run whose style_id is not in the emitted palette →
asserting the counter/warn fires and the rest of the sheet still exports.
Do NOT change the drop behavior itself (fallback design is a separate
decision).
**Verify**: `cargo test -p xlsx-parser` 0 failures.

## Step 3 (S-M): Batch the per-cell #REF! recompute bridge loop
Evidence: `kernel/src/bridges/compute/compute-bridge.ts:1393-1407`
`_forceRecomputeRefErrorCells` awaits `getCellIdAt` + `getFormula` PER CELL
per sheet on every sheet deletion. Extend the Rust side: make
`find_cells_by_formula` (find it in `storage/engine/queries.rs` — the bridge
calls `findCellsByFormula`) return `(row, col, formula_text)` triples (or add
a sibling `find_cells_with_formulas_by_text`), regenerate the TS bridge
(`kernel/src/bridges/compute/compute-bridge.gen.ts` is generated — find the
codegen command in the repo: grep package.json scripts for "gen"; if
generation isn't reproducible locally, STOP this step and report), and
collapse the TS loop to one call per sheet + one `setCellValuesParsed`.
**Verify**: `pnpm --filter @mog-sdk/kernel typecheck`; kernel sheet-deletion
tests (`grep -rln "forceRecompute\|RefError" kernel/src --include=*test*`);
`cargo test -p compute-core --lib queries` (or the module the fn lives in).

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-o`, branch
`opt/round4-small-wins`, prefix `sapiex-patches:`, no push.

## STOP conditions
Per step: two failed attempts → skip, note, continue. Step 3: bridge codegen
not runnable locally → STOP that step (hand-editing .gen.ts is forbidden).
