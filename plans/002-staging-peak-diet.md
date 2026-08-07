# Plan 002: Cut the transient memory peak of full deferred hydration (staging)

> **Executor instructions**: Follow step by step; run every verification and
> confirm expected results. On any STOP condition, stop and report. Update
> your row in `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- compute/core/src/storage/engine/construction/deferred.rs`
> Plan 001 legitimately edits the materialize path in this file — expect that.
> Compare THIS plan's excerpts (staging region, lines ~500-815) against live
> code; on mismatch there, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: plans/001-sheet-switch-clone-elimination.md (same file; land 001 first)
- **Category**: perf
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

`stage_deferred_hydration` (the full-workbook hydration that runs on
read/write documents after first paint) holds, at its peak, roughly **three
full copies of the workbook plus two Yrs documents simultaneously**: the
retained `DeferredHydrationData` (parse output + snapshot + raw bytes), the
freshly parsed `full_parse_output`, the freshly built `full_snap`, a
`full_snap.clone()` handed to ComputeCore init, per-sheet cloned range
payloads, and both the old and new Yrs storages. On wasm32 this transient
peak — not the steady state — is what turned a 1.4M-cell workbook into a
4GiB OOM trap. This plan removes the avoidable copies inside staging without
changing the staging/commit contract.

## Current state

Excerpts verified at `60b6ee3e`, all in
`compute/core/src/storage/engine/construction/deferred.rs`:

```rust
// :652 — per-sheet clone of every sheet's compact range payloads:
                range_data_per_sheet.push(snap_sheet.ranges.clone());

// :768 and :772 — full workbook snapshot cloned into ComputeCore init
// (two variants; both clone; full_snap stays live afterwards):
                new_compute.init_from_snapshot_minimal(&mut new_mirror, full_snap.clone())?;
                new_compute.init_from_snapshot_no_recalc(&mut new_mirror, full_snap.clone())?;
```

Contract you must preserve (comment at `:498-503`): staging takes
`engine: &YrsComputeEngine` (shared ref) so a FAILED staging leaves the engine
untouched and retryable. Do not change that signature; do not free
`engine.deferred_hydration` contents from inside staging (that was considered
and rejected — it breaks retryability).

Also in scope, small: profile counters exist in this region
(`profile.counter("sheets", ...)` etc. around `:655-665`). The identity
allocation loop near `:545-575` has counters for allocated cells; we add two
more for observability (identity_rows/identity_cols) — they feed a later
measurement decision on sparse row identity.

`compute/core/src/storage/infra/hydration/sheet/identity.rs` (~:24-38)
computes `sheet_identity_extent` — do not modify it; only count what it
produces.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Focused tests | `cargo test -p compute-core --lib test_deferred_xlsx_import` | all pass |
| Full lib | `cargo test -p compute-core --lib` | all pass |

## Scope

**In scope**:
- `compute/core/src/storage/engine/construction/deferred.rs` (staging region
  only, ~`:495-815`)
- Signatures of `init_from_snapshot_minimal` / `init_from_snapshot_no_recalc`
  (to take the snapshot by value at last use, or by reference — choose the
  cheaper diff) and mechanical fixes at their call sites
- `compute/core/src/storage/engine/tests/test_deferred_xlsx_import/` (tests)

**Out of scope**:
- The `sapiex_skip_yrs` branch (plan 003 owns it; if plan 003 already landed
  in this worktree, the branch is gone — fine either way)
- `materialize_deferred_sheet_inner` (plan 001 owns it)
- The staging function signature (`&YrsComputeEngine`) and the retry contract
- `raw_xlsx_bytes` lifetime (rejected: cannot be freed from `&self`)
- Yrs hydration internals (`hydrate_from_parse_output_with_ranges` body)

## Git workflow

Branch `opt/deferred-diet` (this worktree), commit prefix `sapiex-patches:`,
do not push.

## Steps

### Step 1: Eliminate the `full_snap.clone()` at ComputeCore init

Reorder so the ComputeCore init is the LAST consumer of `full_snap`, then
pass ownership (`full_snap` moved, no clone). Read the uses between `:726-772`
first: the three index builders (`:726-742`) take `&full_snap` — they can run
before init unchanged; check for any use AFTER `:772` (e.g.
`snapshot_id_high_water_mark(&full_snap)` at ~`:745`) and move those reads
before the init too. If a post-init consumer cannot be reordered, change init
to take `&WorkbookSnapshot` instead and delete the clone that way.

**Verify**: `cargo check --workspace --locked` → exit 0;
`cargo test -p compute-core --lib test_deferred_xlsx_import` → all pass.

### Step 2: Stop cloning per-sheet range payloads

`:652` `range_data_per_sheet.push(snap_sheet.ranges.clone());` — convert
`range_data_per_sheet` to hold references (`&[...]` with a lifetime local to
the staging block) or, if the consumer (`hydrate_from_parse_output_with_ranges`
at `:694+`, parameter `&range_data_per_sheet`) needs owned data internally,
restructure so it borrows. STOP if the hydration function stores these beyond
the call (grep its body for moves of the parameter).

**Verify**: `cargo check --workspace --locked` → exit 0.

### Step 3: Add identity-extent counters (observability rider)

Next to the existing allocation counters (~`:568-573`), add:
`profile.counter("identity_rows", ...)` and
`profile.counter("identity_cols", ...)` summed across sheets from the
allocations just built. This is 3-6 lines; match the existing
`PhaseTimer`/counter style exactly.

**Verify**: `cargo test -p compute-core --lib test_deferred_xlsx_import` →
all pass (counters are passive).

### Step 4: Peak-shape regression test

Add a test asserting the reordering didn't break equivalence: full deferred
open + `complete_deferred_hydration` on a small fixture produces the same
sheet/cell content as a direct (non-deferred) open (a comparison test likely
already exists in `test_deferred_xlsx_import/` — extend or model on it).

**Verify**: `cargo test -p compute-core --lib` → all pass.

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] `cargo test -p compute-core --lib` exits 0
- [ ] `grep -n "full_snap.clone()" compute/core/src/storage/engine/construction/deferred.rs` → no matches
- [ ] `grep -n "ranges.clone()" compute/core/src/storage/engine/construction/deferred.rs` → no match at the staging site
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- Any post-init consumer of `full_snap` cannot be reordered AND init cannot
  take a reference.
- `hydrate_from_parse_output_with_ranges` stores the range-data parameter
  beyond the call.
- The staging retry contract (comment `:498-503`) would need to change.
- Persistent test failure after two attempts.

## Maintenance notes

- ARCH follow-up (held, not this plan): true streaming per-sheet hydration
  into the live doc (one transaction per sheet, observer suppressed) —
  recorded in README as a sol-level design task gated on measurements.
- Reviewers: check Step 1's reorder against BOTH init variants (`minimal` and
  `no_recalc` — skip/non-skip paths) if plan 003 hasn't deleted the skip yet.
