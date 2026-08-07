# Plan 001: Eliminate the O(N²) clones on deferred sheet materialization

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update your row in `plans/README.md` (absolute path:
> `/Users/vish/Repos/analyst/mog/plans/README.md`).
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- compute/core/src/storage/engine/construction/deferred.rs compute/core/src/storage/infra/hydration/`
> If any in-scope file changed since 60b6ee3e, compare the "Current state"
> excerpts against live code; on mismatch treat as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none
- **Category**: perf
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

Mog opens large XLSX workbooks "deferred": only the landing sheet is parsed
into engine state; other sheets materialize on demand when the user clicks
their tab (`materialize_deferred_sheet`). That per-sheet path currently
**clones the entire cumulative all-sheets ParseOutput and WorkbookSnapshot on
every single sheet switch**, and separately clones every sheet's ID vectors
(u128 CellIds — ~22MB per copy at 1.4M cells) up to three times. Browsing a
28-sheet workbook costs O(N²) total bytes cloned and a 2x transient memory
spike per click, inside a wasm32 4GiB address space that this exact workbook
already OOM'd once. This plan removes the clones without changing behavior.

## Current state

All excerpts verified against `60b6ee3e`.

`compute/core/src/storage/engine/construction/deferred.rs` — the materialize path:

```rust
// deferred.rs:389 — full clone of the ALL-SHEETS parse output, every call:
    let mut cumulative_parse = deferred.parse_output.clone();

// deferred.rs:466 — second full clone (snapshot) handed to mirror init:
    compute.init_from_snapshot_viewport_only(&mut mirror, cumulative_snap.clone())?;

// deferred.rs:487-489 — the mutated clones are stored back; next call clones
// an even larger structure:
    deferred.parse_output = cumulative_parse;
    deferred.allocations = allocations;
    deferred.workbook_snap = cumulative_snap;
```

ID-vector clone sites (each `Vec<CellId>`/`Vec<RowId>`/`Vec<ColId>`, CellId is
`u128` = 16 bytes):

- `deferred.rs:83-85` (fast path), `deferred.rs:419-421` (materialize),
  `deferred.rs:582-584` (staging):
  ```rust
  m.cell_ids.push(alloc.cell_ids.clone());
  m.row_ids.push(alloc.row_ids.clone());
  m.col_ids.push(alloc.col_ids.clone());
  ```
- `compute/core/src/storage/infra/hydration/import.rs:454-456` — the same
  triple-clone inside `hydrate_from_parse_output_with_ranges`.

The shared struct being populated:
`compute/core/src/storage/infra/hydration/mod.rs:84-106` — `HydrationIdMap`
holds `cell_ids: Vec<Vec<CellId>>`, `row_ids: Vec<Vec<RowId>>`,
`col_ids: Vec<Vec<ColId>>`. The source of the data,
`SheetIdAllocation`, is in
`compute/core/src/storage/infra/hydration/sheet/identity.rs` (~line 100+),
holding the same vectors by value.

Repo conventions: plain-Rust engine code, no `unsafe`, errors are
`ComputeError`; tests for this area live in
`compute/core/src/storage/engine/tests/test_deferred_xlsx_import/` — model any
new test on `bootstrap_rendering.rs` there.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck workspace | `cargo check --workspace --locked` | exit 0 |
| Focused tests | `cargo test -p compute-core --lib test_deferred_xlsx_import` | all pass (35+ tests) |
| Broader engine tests | `cargo test -p compute-core --lib storage::engine` | all pass |

## Scope

**In scope** (only files you may modify):
- `compute/core/src/storage/engine/construction/deferred.rs`
- `compute/core/src/storage/infra/hydration/mod.rs` (HydrationIdMap fields)
- `compute/core/src/storage/infra/hydration/import.rs` (clone sites + any
  compiler-required consumer adjustments)
- `compute/core/src/storage/infra/hydration/sheet/identity.rs` /
  `sheet/allocation.rs` (only if converting to `Arc<[T]>` requires the
  producer to build Arcs)
- Any file the compiler forces you to touch because a `Vec<CellId>` became
  `Arc<[CellId]>` — mechanical deref/iter fixes only
- `compute/core/src/storage/engine/tests/test_deferred_xlsx_import/` (add test)

**Out of scope** (do NOT touch):
- The `sapiex_skip_yrs` branch at `deferred.rs:667-746` (plan 003 owns it)
- `stage_deferred_hydration`'s `full_snap.clone()` at `:768`/`:772` (plan 002)
- Anything under `kernel/` or `apps/` (TS side)
- Yrs schema / hydration write logic

## Git workflow

- Work on the current branch of this worktree (`opt/deferred-diet`).
- One commit per step group, message prefix `sapiex-patches:` (matches
  `git log` convention, e.g. `sapiex-patches: materialize deferred read-only sheets on demand`).
- Do NOT push.

## Steps

### Step 1: Replace the parse_output clone with move-and-put-back

In `materialize_deferred_sheet_inner` (`deferred.rs`), replace
`let mut cumulative_parse = deferred.parse_output.clone();` with
`let mut cumulative_parse = std::mem::take(&mut deferred.parse_output);`.
`ParseOutput` derives `Default` — verify with
`grep -n "derive" domain-types/src/parse_output.rs | head -5`; if it does NOT
derive `Default`, add `#[derive(Default)]`-compatible manual construction or
implement `Default` (STOP if that is non-trivial).
The put-back at `:487` (`deferred.parse_output = cumulative_parse;`) already
restores it. IMPORTANT: audit every early-`return`/`?` between the take and
the put-back — on any early exit the parse_output must be restored (wrap the
body or restore before each early return; the enclosing
`materialize_deferred_sheet` at `:326-339` already has a take/put-back pattern
for the whole `DeferredHydrationData` you can mirror).

**Verify**: `cargo check --workspace --locked` → exit 0.

### Step 2: Same treatment for workbook_snap

`cumulative_snap` is built from `deferred.workbook_snap` (read the ~30 lines
above `:466` to find its construction — it is also clone-based). Apply the
same take/mutate/put-back so no full copy of the snapshot is made.

**Verify**: `cargo check --workspace --locked` → exit 0.

### Step 3: Stop cloning the snapshot into mirror init

`deferred.rs:466`:
`compute.init_from_snapshot_viewport_only(&mut mirror, cumulative_snap.clone())?;`
Change `init_from_snapshot_viewport_only` to take `&WorkbookSnapshot` instead
of by value, and fix its internals (it may clone what it needs internally —
that is fine if it clones per-sheet slices, not the whole snapshot; if its
internals fundamentally require ownership of the whole snapshot, STOP and
report what it consumes). There is a second caller at `deferred.rs:219`
(`workbook_snap.clone()`) — convert it too and delete that clone.

**Verify**: `cargo check --workspace --locked` → exit 0, and
`cargo test -p compute-core --lib test_deferred_xlsx_import` → all pass.

### Step 4: Share ID vectors via Arc instead of triple-cloning

Change `HydrationIdMap` fields (`hydration/mod.rs:88-106`) from
`Vec<Vec<CellId>>` / `Vec<Vec<RowId>>` / `Vec<Vec<ColId>>` to
`Vec<Arc<[CellId]>>` / `Vec<Arc<[RowId]>>` / `Vec<Arc<[ColId]>>`, and change
`SheetIdAllocation` to hold `Arc<[T]>` for the same three (its builder in
`sheet/allocation.rs` builds a `Vec` then converts once via `.into()`).
Update the seven clone sites listed in "Current state" to `Arc::clone`.
Let the compiler drive the remaining consumer fixes (indexing and iteration
over `Arc<[T]>` works via deref; `.push` producers need `.into()`).

**Verify**: `cargo check --workspace --locked` → exit 0;
`cargo test -p compute-core --lib` → all pass (full lib suite).

### Step 5: Regression test

Add a test in `test_deferred_xlsx_import/` (model on
`bootstrap_rendering.rs`): open a small multi-sheet fixture deferred,
materialize sheets one by one, and assert (a) values render per existing
patterns, and (b) `Arc::strong_count` on one shared id vec is >1 after
materialization (proves sharing, not copying).

**Verify**: `cargo test -p compute-core --lib test_deferred_xlsx_import` →
all pass including the new test.

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] `cargo test -p compute-core --lib` exits 0
- [ ] `grep -n "parse_output.clone()" compute/core/src/storage/engine/construction/deferred.rs` → no match in `materialize_deferred_sheet_inner`
- [ ] `grep -n "cell_ids.clone()" compute/core/src/storage/engine/construction/deferred.rs compute/core/src/storage/infra/hydration/import.rs` → no matches (Arc::clone allowed)
- [ ] No files outside Scope modified (`git status`)
- [ ] `plans/README.md` row updated

## STOP conditions

- `ParseOutput` or `WorkbookSnapshot` cannot cheaply implement `Default`.
- `init_from_snapshot_viewport_only` fundamentally requires owning the whole
  snapshot.
- Early-return audit in Step 1 finds a path where restoring state is
  ambiguous.
- Any test failure that persists after two fix attempts.
- A change appears to require touching Yrs write logic or TS code.

## Maintenance notes

- Plan 002 edits `stage_deferred_hydration` in the same file — land this
  first (it is scoped to the materialize path; 002 rebases trivially).
- Reviewers: scrutinize the early-return restoration in Step 1 — losing
  `parse_output` on an error path would break later materializations.
- Deferred follow-up (documented in README): LRU eviction of materialized
  sheets requires kernel-side tracker coordination and is NOT part of this plan.
