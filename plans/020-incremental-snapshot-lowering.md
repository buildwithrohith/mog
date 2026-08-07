# Plan 020: Incremental snapshot lowering — stop re-lowering every visited sheet

> Follow steps in order; update row 020 in plans/README.md when done (the one
> file writable outside your worktree).
> **Drift check**: base `02403d9b`;
> `git diff --stat 02403d9b..HEAD -- compute/core/src/import/parse_output_to_snapshot/ compute/core/src/storage/engine/construction/deferred.rs` empty in YOUR worktree.

## Status
P1 / L / Risk MED-HIGH / Planned at `02403d9b`. This is the audit-flagged gap
the 008 design deliberately kept ("reuse the cumulative sparse rebuild") and
the last O(cumulative) cost on every sheet materialization.

## Why (verified evidence)
`materialize_deferred_sheet_inner` and `hydrate_deferred_sheet_inner`
(`construction/deferred.rs`) both call
`parse_output_to_workbook_snapshot(&cumulative_parse, Some(&id_map), &mut allocator)`
over the ENTIRE cumulative parse on EVERY materialization. Plan 017 scoped the
index/mirror rebuild to one sheet, but this snapshot lowering still reruns
`sheet_lowering::convert_sheets` (per-cell hex UUID alloc + map builds,
`import/parse_output_to_snapshot/sheet_lowering.rs:31-146`) and
`classifier::classify_sheet_ranges` (per-cell hash/sort/byte-encode,
`classifier.rs:89-153`) for every ALREADY-LOWERED sheet
(`mod.rs:64-151` has no incremental mode). Measured consequences: full-browse
wasm arena still ratchets to ~4,017MB (arena never shrinks; each click's
transient full-relowering sets the floor), and the native import scaffolding
peak stays ~10.7GB.

## The change
Give the deferred path a persistent, incrementally-updated snapshot instead of
a rebuilt one:
1. Retain `cumulative_snap: WorkbookSnapshot` in the deferred state (it
   already stores one — `deferred.workbook_snap`) as the AUTHORITATIVE lowered
   form. On materializing sheet k, lower ONLY sheet k
   (`parse_output_to_workbook_snapshot` gains a single-sheet mode, or a new
   `lower_single_sheet(parse, sheet_index, id_map, allocator) -> SheetSnapshot`
   extracted from `convert_sheets`' per-sheet body) and REPLACE
   `workbook_snap.sheets[k]`, leaving other sheets' lowered data untouched.
2. Workbook-LEVEL lowering outputs (names, tables, pivots, cross-sheet range
   metadata produced by `mod.rs:64-151` around the sheet loop): identify each
   one; recompute only those whose inputs can change when one sheet's parse is
   replaced (READ the driver carefully; list your findings in the commit
   message). If any output is genuinely global-recompute-only and cheap,
   recompute it; if expensive, STOP and report the specific output.
3. `id_map`/allocator: sheet k's identity allocation already reuses prior
   allocations (round-1/015 machinery, `SharedIdAllocator`); the single-sheet
   lowering must consume the same allocation for k that the full pass would
   have produced — the existing tests
   (`test_deferred_xlsx_import/identity_allocation.rs`, `existing_sheet_hydration.rs`)
   are your safety net.
4. Apply to BOTH callers (materialize + hydrate paths in deferred.rs) and to
   `stage_deferred_hydration`'s full pass ONLY if trivially compatible —
   otherwise leave staging's one-shot full lowering unchanged (it runs once,
   not per click; NOT the target).

## Verify (each step)
`cargo check --workspace --locked`;
`cargo test -p compute-core --lib test_deferred_xlsx_import` (all pass — the
bootstrap_rendering formats tests + existing_sheet_hydration idempotence
cover switch correctness); full `cargo test -p compute-core --lib` → 0
failures; `cargo test -p xlsx-parser` → 0 failures.
New test: materialize sheets A,B,C on a 3-sheet fixture with a `#[cfg(test)]`
counter proving sheet A is lowered exactly ONCE across the three
materializations (model on plan 017's builder-count test in
bootstrap_rendering.rs).

## Scope
**In**: `import/parse_output_to_snapshot/` (single-sheet mode),
`construction/deferred.rs` (the two call sites + retained snapshot update),
tests. **Out**: `stage_deferred_hydration` full pass (unless trivial),
scheduler, mirror, Yrs hydration internals, parser crate.

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-n`, branch
`opt/incremental-lowering`, commit per step, prefix `sapiex-patches:`, no push.

## STOP conditions
- A workbook-level lowering output can't be incrementally maintained and is
  expensive to recompute per materialization — report which and its cost.
- Identity divergence: single-sheet lowering produces different ids than the
  full pass for the same input (test catches it) — report, do not paper over.
- Persistent failure after two attempts.
