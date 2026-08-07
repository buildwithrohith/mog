# Plan 024: Merge the mirror's pos_to_id / id_to_pos double map

> **Drift check**: base `c7f9f39c`;
> `git diff --stat c7f9f39c..HEAD -- compute/core/src/mirror/` empty in YOUR
> worktree.

## Status
P2 / M-L / Risk MED. Planned at `c7f9f39c`.

## Why (verified evidence)
`mirror/types.rs:159-161`: every SheetMirror carries BOTH
`pos_to_id: FxHashMap<SheetPos, CellId>` and
`id_to_pos: FxHashMap<CellId, SheetPos>`, each with a per-cell entry
(~40B+overhead per entry per map; on the 1.4M-cell fixture the second map is
tens of MB per workbook and every insert/delete/shift must touch both — a
drift hazard). The read paths are ALREADY encapsulated:
`position_of()` (types.rs:379) and `cell_id_at()` (types.rs:393) are the
accessors; virtual/range-backed IDs already have non-map fallbacks.

## The change
Make `pos_to_id` the single owned map and replace `id_to_pos` with a derived
structure that is cheaper than a second full FxHashMap. Decide between:
(a) lazy per-sheet reverse map rebuilt on first `position_of` after a bulk
mutation epoch (invalidation counter), or (b) a compact
`FxHashMap<CellId, SheetPos>` kept but keyed by the u128 CellId with no
change — REJECTED, that is the status quo; or (c) since CellId is derived
and dense per sheet in the common import path, an index-vector keyed by
allocation order. Investigate (a) vs (c) by reading how often `position_of`
is called on hot paths (grep callers; scheduler/undo are the hot ones) and
pick with a one-paragraph justification in the commit message.
AUDIT every direct field access to `id_to_pos`/`pos_to_id` outside the two
accessors first (`grep -rn "id_to_pos\|pos_to_id" compute/core/src/`) and
route them through accessors in a preparatory commit.

## Verify
Preparatory commit: `cargo test -p compute-core --lib` 0 failures with pure
accessor routing (no representation change). Main commit: same suite 0
failures; structural ops (insert/delete rows/cols, sheet copy) tests pass;
add a test asserting position_of/cell_id_at consistency after an
insert-delete-shift sequence on a mixed cell+range sheet (model on
mirror/tests/ structural tests). State the estimated per-cell memory saving
in the commit message (sizeof math is fine, no heap profiler required).

## Scope
**In**: compute/core/src/mirror/** and its tests. **Out**: persisted Yrs
schema (plan 023 owns storage/), kernel TS, bridge surface.

## Git workflow
Worktree /Users/vish/Repos/analyst/mog-opt-q, branch opt/mirror-bimap,
commit per step, prefix `sapiex-patches:`, no push. Only external writable
file: plans/README.md (row 024). Never run git in the main repo.

## STOP conditions
- A hot path measurably needs O(1) id→pos and neither option keeps it —
  report with the call site instead of regressing.
- Two failed attempts.
