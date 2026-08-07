# Plan 003: Remove the empty-Yrs landmine (skip branch) and harden export/sync guards

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- compute/core/src/storage/engine/construction/deferred.rs compute/core/src/storage/engine/export.rs compute/core/src/storage/engine/sync_bridge.rs`
> Plans 001/002 legitimately touched deferred.rs (materialize + staging
> reorder). Verify THIS plan's excerpts (the skip branch, the guard, commit)
> still match; on mismatch there, STOP.

## Status

- **Priority**: P0 (correctness — silent data loss)
- **Effort**: S-M
- **Risk**: LOW-MED
- **Depends on**: 001, 002 (same file; land in order)
- **Category**: bug
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

Commit `bb67ba70` added a stopgap: above 600,000 cells, full Yrs hydration is
skipped, but the pipeline still **commits an EMPTY Yrs document and then
clears the one guard that blocks exports**. Terminal state: `export_to_xlsx`
returns a valid workbook with zero sheets (silent), sync
checkpoints persist the empty doc as authoritative (silent), and all
formatting reads collapse to defaults. It is reachable today on any
read/write document >600k cells via `complete_deferred_hydration`, and via
`apply_structure_change_bridge` (row/col insert/delete) which has **no gate at
all**. The skip no longer serves its original purpose: the read-only path
(commit `9cb276f3`/`60b6ee3e`) never runs full hydration, and the follow-up
commit message records that the skip did not prevent the OOM anyway. Delete
it; make staging fail loudly instead; add defensive guards.

## Current state

Verified at `60b6ee3e` (deferred.rs line numbers may have shifted slightly
after plans 001/002 — re-locate by content):

```rust
// deferred.rs:668-681 — the stopgap comment + constant:
        // sapiex-patches: the Yrs CRDT write costs >4GB of wasm32 linear memory
        // ... Above this threshold, skip the Yrs hydration entirely ...
        const SAPIEX_YRS_HYDRATION_MAX_CELLS: u64 = 600_000;
        let sapiex_total_cells: u64 = full_parse_output...
        let sapiex_skip_yrs = sapiex_total_cells > SAPIEX_YRS_HYDRATION_MAX_CELLS;

// deferred.rs:684-692 — new_storage created; when skip: NEVER hydrated
// (stays a truly empty Yrs doc), id_map defaulted:
        let mut new_storage = YrsStorage::new();
        let id_map = if sapiex_skip_yrs {
            tracing::warn!(...);
            crate::storage::infra::hydration::HydrationIdMap::default()
        } else { /* real hydration */ };

// deferred.rs:950 — commit unconditionally clears the ONLY export guard:
    engine.deferred_hydration = None;
```

```rust
// export.rs:200-209 — the guard that gets cleared:
    fn require_all_sheets_materialized(&self, operation: &str) -> Result<(), ComputeError> {
        if self.deferred_hydration.is_some() {
            return Err(ComputeError::InvalidInput { message: format!(
                "{operation} requires deferred XLSX hydration to complete before reading all sheets") });
        }
        Ok(())
    }
```

```rust
// structure_change.rs:15 — gate-free entry that reaches the terminal state:
        self.complete_deferred_hydration_for_structure_change()?;
```

`sync_bridge.rs` — `encode_state_vector` (:~91), `current_state_vector`
(:~103), `encode_diff` (:~108) have no materialization/emptiness guard.

Convention: errors are `ComputeError::InvalidInput { message }`; tests live in
`compute/core/src/storage/engine/tests/` (model on
`test_deferred_xlsx_import/bootstrap_rendering.rs`).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Focused | `cargo test -p compute-core --lib test_deferred_xlsx_import` | all pass |
| Export tests | `cargo test -p compute-core --lib export` | all pass |
| Full lib | `cargo test -p compute-core --lib` | all pass |

## Scope

**In scope**:
- `compute/core/src/storage/engine/construction/deferred.rs` — delete the
  skip branch/constant; add the staging emptiness bail-out
- `compute/core/src/storage/engine/export.rs` — extend guard
- `compute/core/src/storage/engine/sync_bridge.rs` — add guard
- `compute/core/src/storage/engine/tests/` — new tests

**Out of scope**:
- kernel/ and apps/ TS (read-only gating there already exists and stays)
- `structure_change.rs` logic itself (its call into completion becomes safe
  once staging bails; do not add TS-visible behavior changes there)
- Any partial-hydration redesign (held follow-up)

## Git workflow

Branch `opt/deferred-diet`, commit prefix `sapiex-patches:`, do not push.

## Steps

### Step 1: Delete the skip branch

Remove the constant, the cell-count computation, and the `if sapiex_skip_yrs`
arm; keep the real-hydration arm unconditionally. Also remove the now-dead
skip-only index-construction variant if one exists nearby (compare `:726-753`
region: if there are two index paths keyed on the skip flag, keep the
non-skip one).

**Verify**: `cargo check --workspace --locked` → exit 0;
`grep -rn "SAPIEX_YRS_HYDRATION_MAX_CELLS\|sapiex_skip_yrs" compute/ --include=*.rs`
→ no matches.

### Step 2: Make staging refuse to produce an empty replacement doc

At the end of `stage_deferred_hydration`, before returning the completion:
if `full_parse_output.sheets` is non-empty but the new storage's sheet order
is empty (`new_storage.sheet_order().is_empty()`), return
`Err(ComputeError::InvalidInput { message: "deferred hydration produced an empty Yrs document; refusing to commit (would clear the export guard over nothing)".into() })`.
This is the fallible-staging contract the surrounding comments already
promise: on Err, the engine keeps its guard and stays retryable.

**Verify**: `cargo check --workspace --locked` → exit 0.

### Step 3: Defensive guards on export + sync

- `export.rs` `require_all_sheets_materialized`: after the existing
  `deferred_hydration` check, add: if `self.stores.storage.sheet_order().is_empty()`
  while the mirror reports at least one sheet (find the mirror's sheet-count
  accessor via `grep -n "fn sheet" compute/core/src/mirror/cell_mirror.rs | head`),
  return an `InvalidInput` error naming the inconsistency.
- `sync_bridge.rs`: in `encode_diff` (it already returns `Result`), add the
  same emptiness-vs-mirror inconsistency check and error. For the two
  infallible encoders (`encode_state_vector`, `current_state_vector`), do NOT
  change signatures; leave them (the diff path is what providers persist).

**Verify**: `cargo test -p compute-core --lib export` → all pass.

### Step 4: Tests

1. Unit test the staging bail-out: construct the smallest engine fixture that
   enters staging with a stubbed/emptied hydration result if feasible; if the
   internal seam is not test-reachable without large fixtures, instead test
   the guards directly: build an engine whose storage has empty sheet_order
   and non-empty mirror (test helpers in `storage/tests/` may construct
   engines — search `grep -rn "fn test_engine\|fixture" compute/core/src/storage/engine/tests/ | head`),
   assert export and `encode_diff` return errors containing "empty".
2. Structure-change path: on a small (below-nothing-special) deferred
   workbook, call the structural path
   (`complete_deferred_hydration_for_structure_change` via its public bridge
   if reachable in tests) and assert post-commit `sheet_order()` is
   non-empty and export succeeds.

**Verify**: `cargo test -p compute-core --lib` → all pass, incl. new tests.

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] `cargo test -p compute-core --lib` exits 0
- [ ] grep for the constant/flag returns nothing (Step 1 check)
- [ ] New tests exist and pass (empty-doc refusal + structural path)
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- Deleting the skip branch reveals a THIRD consumer of the flag beyond the
  hydration arm and index-path selection.
- The mirror has no cheap sheet-count accessor for the consistency guard.
- The structural-path test cannot reach the completion function through any
  test-visible seam (report what seams exist instead of forcing one).
- Persistent failure after two attempts.

## Maintenance notes

- Consequence to document in the commit message: read/write documents
  >600k cells now attempt REAL full hydration; on wasm32 that can still OOM
  (pre-existing condition, previously silent corruption instead). Native
  (server) builds are unaffected. The honest failure is the Err from Step 2 /
  the OOM trap, both visible — silent empty exports are gone.
- Held follow-up (README): first-class partial-hydration state + per-sheet
  incremental Yrs hydration (design task).
