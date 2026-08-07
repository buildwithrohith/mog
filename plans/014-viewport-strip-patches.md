# Plan 014: Strip-based row/col format viewport patches (kill the per-cell hash set)

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: base `80f56679`. Plan 013 lands FIRST in this
> worktree (same lane, same file `viewport/patches.rs`); start 014 only
> after 013's done criteria pass.

## Status

- **Priority**: P2
- **Effort**: S-M
- **Risk**: LOW-MED
- **Depends on**: 013 (same lane/file ordering only)
- **Category**: perf
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

`compute/core/src/storage/engine/viewport/patches.rs:276-398`
(`produce_row_col_format_viewport_patches`): for every changed row, it
inserts EVERY visible column position into an `FxHashSet<(u32,u32)>` one
tuple at a time (and symmetrically for changed cols), then drains and sorts:

```rust
        let mut positions: FxHashSet<(u32, u32)> = FxHashSet::default();
        for (_viewport_id, bounds) in self.viewport.viewports_for_sheet(sheet_id) {
            for &row in rows {
                if row < bounds.start_row || row > bounds.end_row { continue; }
                for col in bounds.start_col..=bounds.end_col {
                    positions.insert((row, col));               // per-cell insert
                }
            }
            ...
```
The inputs are provably RECTANGULAR STRIPS (full contiguous col range per
row, full contiguous row range per col). Formatting one row on a
100-visible-column viewport does 100 hash inserts + drain + sort; a
row-height change touching 50 rows does 5,000. The set exists only to dedup
overlap (a) between multiple registered viewports and (b) between the
row-strips and col-strips at their crossings.

Positive in-repo contrast: `viewport/mod.rs` `compute_delta_strip`
(~:262-387,:427+) already computes batched rectangular strips — this
function predates that idiom.

## The change

Represent the work as strips, dedup at strip level:
- Row strips: `(row, col_start..=col_end)` per (changed row × viewport
  bounds intersection); col strips symmetric.
- Merge overlapping strips per row (viewports overlap rarely; a simple
  sort-and-merge of ranges per row key).
- Row-strip × col-strip crossings: acceptable to emit the crossing cell in
  BOTH only if the downstream consumer dedups; read what the positions feed
  (~:308-398: the sorted Vec drives per-position format fetch +
  serialize_multi_viewport_patches) — if the wire format tolerates
  duplicate positions, note it and keep it simple by removing col positions
  already covered by a row strip (membership test against the row-strip
  ranges: binary search per col position; cheap). If it does NOT tolerate
  duplicates, that same membership subtraction is REQUIRED — state which in
  the commit message.
- Keep the final output ORDER identical to today (the existing
  `sort_unstable` order) so serialized patches are byte-comparable.

The function's output (the serialized patch bytes) must be BYTE-IDENTICAL
to today's for the same inputs — that is the review bar and the test.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Patch/viewport suites | `cargo test -p compute-core --lib viewport` and `--lib patches` | all pass |
| Full engine | `cargo test -p compute-core --lib` | zero failures (post-009) or documented baselines |

## Scope

**In scope**: `produce_row_col_format_viewport_patches` in
`viewport/patches.rs` + a byte-equivalence test.

**Out of scope**: `produce_observer_format_patches` (~:793-836 — its
entry()-based maps are keyed by genuinely scattered doc changes, not strips;
documented as a non-target); `merge_patch_binaries`; the wire format.

## Git workflow

Same worktree/branch as 013 (`/Users/vish/Repos/analyst/mog-opt-g`,
`opt/cf-and-patches`), separate commit, prefix `sapiex-patches:`, no push.

## Steps

1. Byte-equivalence test FIRST: capture today's output bytes for (a) 3 rows
   changed on one viewport, (b) rows+cols with crossings, (c) two
   overlapping viewports, (d) empty intersection. Commit passing on old code.
2. Rewrite with strips; the Step-1 test must pass unchanged.
3. Full suites.

## Done criteria

- [ ] Step-1 test green on BOTH old and new implementation (bytes identical)
- [ ] No `FxHashSet<(u32, u32)>` in the function
- [ ] Full suites at expected outcomes; README row updated

## STOP conditions

- The wire format turns out position-order-sensitive in a way the strip
  rewrite can't reproduce exactly — report with the differing byte ranges.
- Persistent failure after two attempts.
