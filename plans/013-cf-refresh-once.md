# Plan 013: Run CF refresh once per mutation, and stop cloning the results map

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: base `80f56679`.
> `git diff --stat 80f56679..HEAD -- compute/core/src/storage/engine/services/cf_cache.rs compute/core/src/storage/engine/recalc_postprocess.rs compute/core/src/storage/engine/viewport/patches.rs`
> in YOUR worktree must be empty before starting.

## Status

- **Priority**: P1 (hot path: runs on EVERY user edit)
- **Effort**: M
- **Risk**: MED (mutation hot path; well-covered by CF tests)
- **Depends on**: none (independent files from 009/010/011/012)
- **Category**: perf
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

On every engine mutation, conditional formatting is fully re-evaluated
TWICE and its per-cell results map is deep-cloned FOUR times:

1. `compute/core/src/storage/engine/recalc_postprocess.rs:28`
   (`prepare_recalc_for_flush`) calls
   `self.refresh_cf_caches_after_recalc(recalc);` — and DISCARDS the
   returned diff.
2. `compute/core/src/storage/engine/viewport/patches.rs:409`
   (`produce_viewport_patches_for_recalc`) calls it AGAIN:
   `let cf_only_changes = self.refresh_cf_caches_after_recalc(recalc);`

These two run back-to-back at essentially every mutation entry point — the
`prepare_recalc_for_flush(...)` + `flush_viewport_patches()` pairing appears
at `cell_bridge.rs:51-52,110-111,151-152,178-179,212-213,238-239`, ~18 sites
in `mutation_dispatch.rs`, `tables.rs:535-536,562-563`,
`features/filters.rs:167-168`, `delegations/batch_cells.rs`,
`delegations/defined_names_print_cells.rs:105-106,310-311`,
`structural/structure_change.rs:97,108`, `structural/merges.rs`,
`delegations/compute_sheets_named.rs:192-193,245-246`.

Inside `refresh_cf_caches_after_recalc`
(`compute/core/src/storage/engine/services/cf_cache.rs:26-116`), per
affected sheet:
```rust
        let old_results: FxHashMap<(u32, u32), CellCFResult> = stores
            .cf_cache.get(sheet_id).map(|e| e.results.clone())   // clone 1
            .unwrap_or_default();
        refresh_cf_cache(stores, mirror, theme_palette, sheet_id); // full CF re-eval
        let new_results: ... = stores
            .cf_cache.get(sheet_id).map(|e| e.results.clone())   // clone 2
            .unwrap_or_default();
```
`CellCFResult` (`compute/core/crates/compute-cf/src/types/result.rs:12-23`)
holds per-cell render styles / data bars / color scales / icons — the clones
are deep. Net per single edit: 2 full CF re-evaluations + 4 map clones per
affected sheet. On a CF-heavy financial sheet this is the dominant per-edit
allocation churn.

## The change (two independent halves)

### Half 1: refresh once per recalc

Make the second computation reuse the first. Preferred shape: give the
engine a small per-flush memo — e.g. `prepare_recalc_for_flush` stores the
returned `cf_only_changes` (keyed by nothing; it's consumed by the very next
`flush_viewport_patches` on the same `&mut self`) in an engine field
(`Option<FxHashMap<SheetId, Vec<(u32,u32)>>>`), and
`produce_viewport_patches_for_recalc` TAKES it (`Option::take`) if present,
only recomputing when absent (defensive fallback for any call order that
skips prepare). This preserves behavior for every call topology, including
any site that calls flush without prepare.

BEFORE choosing the simpler alternative (deleting the call at
`recalc_postprocess.rs:28`), you must enumerate what reads `stores.cf_cache`
between `prepare_recalc_for_flush` and `flush_viewport_patches` across the
listed call sites. If ANYTHING reads it in between (queries, serialization),
the memo approach is required. Document which you chose and why in the
commit message.

### Half 2: eliminate the clones inside the function

Restructure the diff to avoid both deep clones: `Option::take`/`mem::take`
the OLD `results` map out of the cache entry before `refresh_cf_cache`
rebuilds it, then diff old (owned) vs new (borrowed via
`stores.cf_cache.get(sheet_id)`) without cloning the new map. The borrow
constraint that motivated "clone to avoid borrow overlap" disappears once
the old map is owned. The diff logic itself (changed/left/entered cells)
stays byte-identical in behavior.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| CF suites | `cargo test -p compute-core --lib cf` | all pass |
| Viewport patch suites | `cargo test -p compute-core --lib viewport` and `--lib patches` | all pass |
| Mutation round-trips | `cargo test -p compute-core --lib mutation` and `--lib cell_bridge` (adjust to real module test names via `cargo test -p compute-core --lib -- --list \| grep -i cf` first) | all pass |
| Full engine | `cargo test -p compute-core --lib` | zero failures (post-009) or documented baselines only |

## Scope

**In scope**: `services/cf_cache.rs`, `recalc_postprocess.rs`,
`viewport/patches.rs` (the call at :409 only), one engine field if the memo
approach is chosen (wherever `EngineStores`/engine struct fields live —
follow existing field patterns), tests adjacent to CF cache.

**Out of scope**: `refresh_cf_cache` itself (the actual evaluator);
`compute-cf` crate; every mutation entry point (they keep calling
prepare+flush exactly as today — the fix is INSIDE the two callees).

## Git workflow

Worktree `/Users/vish/Repos/analyst/mog-opt-g`, branch `opt/cf-and-patches`
(created for you), commit prefix `sapiex-patches:`. Commit Half 1 and
Half 2 separately. Do not push.

## Steps

1. Enumerate between-call readers of `cf_cache` (grep + read the engine's
   prepare→flush windows at 3 representative sites incl. `cell_bridge.rs:51`
   and one `mutation_dispatch.rs` site). Record findings in the commit
   message. Implement Half 1 accordingly.
   **Verify**: CF + viewport suites pass.
2. Add a regression test proving single evaluation: a counter/flag on
   `refresh_cf_cache` invocations per mutation (a `#[cfg(test)]` counter in
   the engine, or assert via an existing test seam) — one user edit on a
   CF-bearing sheet → exactly ONE `refresh_cf_cache` call.
3. Implement Half 2 (clone elimination).
   **Verify**: CF diff behavior tests pass (changed/entered/left cases —
   extend the existing cf_cache tests if thin).
4. Full engine suite.

## Done criteria

- [ ] One `refresh_cf_cache` execution per mutation (test-proven)
- [ ] Zero `.results.clone()` in `refresh_cf_caches_after_recalc`
- [ ] `cf_only_changes` viewport patches byte-identical in behavior
      (existing patch tests untouched and green)
- [ ] Full suites at expected outcomes; README row updated

## STOP conditions

- Something DOES read `cf_cache` between prepare and flush AND the memo
  approach can't preserve its view — report the reader.
- Any call site flushes without preparing (memo absent) in a way the
  fallback can't serve — report the site.
- Persistent failure after two attempts.

## Maintenance notes

- Reviewers: the risk is a stale memo surviving across TWO mutations if a
  flush is skipped — the memo must be take()-consumed and also cleared at
  the start of each prepare.
