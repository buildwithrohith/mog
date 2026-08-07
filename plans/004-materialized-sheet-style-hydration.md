# Plan 004: Give materialized (non-landing) sheets their real formatting

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- compute/core/src/storage/engine/construction/deferred.rs compute/core/src/storage/infra/hydration/styles.rs`
> Plans 001-003 legitimately edited deferred.rs. Re-locate this plan's
> anchors by content (function `materialize_deferred_sheet_inner`), not line
> number; if the FUNCTION's structure no longer matches the description
> below, STOP.

## Status

- **Priority**: P1 (correctness, user-visible)
- **Effort**: M
- **Risk**: MED
- **Depends on**: 001 (same function was refactored), 003 recommended first
- **Category**: bug
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

When a deferred (read-only) workbook materializes a non-landing sheet on tab
click, the sheet's VALUES come from re-parsing the raw XLSX — but its
FORMATS never arrive: the format cascade reads exclusively from Yrs storage
(`storage/properties/cascade.rs:26-46` — workbook base, column, row, cell
properties are all Yrs reads), the landing sheet is the only sheet ever
hydrated into Yrs on this path, and `materialize_deferred_sheet_inner`
contains no Yrs write at all. The snapshot type it builds carries no style
field (`snapshot-types/src/init.rs` `CellData`: cell_id/row/col/value/formula
only). Observable symptom in the consuming product (screenshot evidence from
2026-08-06): numbers on switched-to sheets render as raw floats
(`04981973.19`, `0.96968...`) where the file defines currency/percent
formats. The fix: when materializing sheet N, also hydrate that sheet's
STYLE data (only) into the live Yrs storage — a small fraction of full
hydration's cost (no per-cell values, no gridIndex writes).

## Current state

Verified at `60b6ee3e`:

- `deferred.rs:326-339` — `materialize_deferred_sheet` takes
  `engine.deferred_hydration`, calls `materialize_deferred_sheet_inner`,
  always restores the deferred guard.
- `deferred.rs:341-491` — `_inner`: parses the selected sheet from
  `raw_xlsx_bytes` via `xlsx_api::parse_selected_sheets`, merges into
  cumulative parse/snapshot, rebuilds indexes/mirror; comment at `:472-475`:
  "The critical sheet's Yrs-backed range formats remain authoritative. The
  newly selected sheet's values/ranges come from the cumulative snapshot."
  No transaction is opened; nothing is written to `engine.stores.storage`.
- Style hydration functions live in
  `compute/core/src/storage/infra/hydration/styles.rs` (e.g.
  `hydrate_cell_styles`, row/col variants, `hydrate_style_palette` — a
  style-id → JSON cache exists at ~`:515-600` sharing `Arc<str>` per
  style_id). Read their signatures first; they were built for the full
  hydration pass and take a transaction + sheet data + id/position inputs.
- Precedent for clearing not-causally-valid update bytes:
  `deferred.rs:919` calls `engine.update_buffer.clear()` at commit with a
  comment that pre-replacement update bytes "are not causally valid".
- The engine's update-buffer observer pushes on EVERY txn commit
  (`update_buffer.rs:202-210`); on the read-only path nothing drains it.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Focused | `cargo test -p compute-core --lib test_deferred_xlsx_import` | all pass |
| Full lib | `cargo test -p compute-core --lib` | all pass |

## Scope

**In scope**:
- `compute/core/src/storage/engine/construction/deferred.rs`
  (`materialize_deferred_sheet_inner` only)
- `compute/core/src/storage/infra/hydration/styles.rs` — ONLY if a thin
  entry point must be added (e.g. a `hydrate_sheet_styles_only(...)` that
  reuses existing internals); do not restructure existing functions
- `compute/core/src/storage/engine/tests/test_deferred_xlsx_import/` (tests)

**Out of scope**:
- Cell VALUE hydration into Yrs (values stay mirror/snapshot-backed)
- gridIndex writes (posToId/idToPos) — explicitly not needed for formats
- kernel/ TS; undo manager redesign; provider protocol

## Git workflow

Branch `opt/deferred-diet`, commit prefix `sapiex-patches:`, do not push.

## Steps

### Step 1: Understand the style-hydration inputs

Read `styles.rs` top-to-bottom. Identify the minimal call set that, for ONE
sheet, writes: style palette entries (workbook-level, idempotency check
required), per-cell `cellProperties`, row styles, col styles, and col style
ranges. Record for each: required inputs (txn, sheet map handle, sheet data,
positions/ids). If per-cell style writes require the Yrs `cells` map entries
to exist (they may write to a properties map keyed by cell hex instead —
verify), note it: cell-keyed properties writes keyed by hex do NOT require
the cell entry itself. STOP if per-cell styles can only attach to existing
Yrs cell entries (then only row/col/range styles are deliverable and the
plan shrinks — report that).

**Verify**: write your findings as a comment block at the top of the new
entry point (Step 2); no build change yet.

### Step 2: Add a style-only hydration entry point

In `styles.rs`, add `pub(crate) fn hydrate_sheet_styles_only(...)` that runs
exactly the calls identified in Step 1 inside a caller-provided transaction,
for one sheet index, reusing the existing `style_only_cache` sharing.
Palette hydration must be idempotent across repeated materializations
(guard: skip style ids already present — check how the full pass detects
existing palette entries, mirror that).

**Verify**: `cargo check --workspace --locked` → exit 0.

### Step 3: Call it from materialize, with buffer hygiene

In `_inner`, after the cumulative snapshot merge succeeds and before index
rebuild: open one `transact_mut` on `engine.stores.storage`, call the Step 2
entry for `sheet_index`, commit. Then, ONLY when
`engine.deferred_hydration`-restore is pending (i.e. always in this function)
call `engine.update_buffer.clear()` with a comment citing the `:919`
precedent: pre-full-hydration update bytes are not causally valid for
providers, and on the read-only path nothing drains the buffer (unbounded
growth otherwise). Suppress undo grouping if the write lands on the undo
stack: check how `structure_change.rs:30` uses
`self.mutation.suppress_guard()` — materialize has access to the engine; use
the same guard around the transaction if applicable at this layer (STOP if
the suppress guard is not reachable from a free function taking
`&mut YrsComputeEngine` — report the seam instead).

**Verify**: `cargo check --workspace --locked` → exit 0;
`cargo test -p compute-core --lib test_deferred_xlsx_import` → all pass.

### Step 4: Test — formats survive materialization

New test (model on `bootstrap_rendering.rs`): fixture workbook where a
NON-landing sheet has a distinctly formatted cell (number format + fill).
Open deferred → materialize that sheet → read the effective format through
the engine's format read path (find the accessor used by existing format
tests: `grep -rn "get_effective_format\|effective_format" compute/core/src/storage/engine/tests/ | head`)
→ assert the number-format string and fill match the file, not defaults.
Also assert repeat materialization of another sheet does not duplicate
palette entries (count palette entries between calls).

**Verify**: `cargo test -p compute-core --lib` → all pass incl. new tests.

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] `cargo test -p compute-core --lib` exits 0
- [ ] New test proves a materialized sheet's number format ≠ default
- [ ] Palette idempotency test passes
- [ ] `engine.update_buffer` cleared after the style txn (grep confirms)
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- Step 1 finds per-cell styles require existing Yrs cell entries AND
  row/col/range styles alone wouldn't fix number formats (report evidence).
- The suppress/undo seam is not reachable from this function.
- Fixture creation for a formatted multi-sheet workbook is not achievable
  with existing test utilities (list what utilities exist).
- Persistent failure after two attempts.

## Maintenance notes

- This intentionally leaves VALUES un-hydrated in Yrs on the read-only path —
  the 4GiB ceiling is per-cell values+gridIndex, not styles. Reviewers should
  verify no step accidentally hydrates cells.
- Future LRU eviction (held) must NOT evict Yrs-hydrated styles (cheap,
  idempotent to keep).
