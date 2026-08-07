# Plan 011: Formula-text dedup — share one allocation across the scheduler's two caches

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: base `80f56679`. Plan 010 lands FIRST in this
> same worktree (same lane); start 011 only after 010's done criteria pass.

## Status

- **Priority**: P2
- **Effort**: S-M
- **Risk**: LOW
- **Depends on**: 010 (same lane, shared file `sheet_lowering.rs`)
- **Category**: perf/memory
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

Formula text is stored in at least six places across the pipeline. The
evaluation hot path stores NONE of it (it evaluates parsed `ASTNode`s and
id-based `IdentityFormula`s), so every text copy serves I/O, reparse, or
display. Two of the copies are pure duplicates seeded from the same tuples:

`compute/core/src/scheduler/mod.rs:180-187`:
```rust
    /// Formula strings stored separately for reparsing when references change.
    formula_strings: FxHashMap<CellId, String>,
    /// Cell-authored formula text used for readback, independent of graph readiness.
    ///
    /// This is deliberately cell-only. `formula_strings` also stores synthetic
    /// variable/named-range formulas used by the graph, so it cannot be the
    /// document identity source for cell formula readback during deferred import.
    cell_formula_text: FxHashMap<CellId, String>,
```
Both are seeded from the same `(CellId, SheetId, String)` tuples
(`scheduler/init.rs` seeding sites ~:78-82,:164-168,:238-241,:309-312,
:344-349,:376-379 and `seed_cell_formula_text` ~:408-412) and regenerated
together (`regenerate_formula_strings_and_cell_formula_text`,
`scheduler/edit.rs:774-857`). Every cell-authored formula string is
heap-allocated TWICE. At financial-model scale (hundreds of thousands of
formulas, average text 20-40 chars) that is tens of MB duplicated, plus
double the allocator traffic on every structural edit's regeneration pass.

The two maps must remain SEPARATE (the doc comment explains why: synthetic
vs cell-authored population), but their VALUES can share one allocation.

## The change

Change both maps to `FxHashMap<CellId, Arc<str>>`. At seed and regeneration
sites, build the `Arc<str>` once and `Arc::clone` into the second map.
Callers that need `&str` get it via deref; callers that need `String` (if
any — compiler will show) get `.to_string()` at the boundary ONLY if that
boundary is cold (display/readback). `FormulaTextProvider`
(`compute/core/src/formula_text.rs:17-62`) reads both maps — adjust its
field types; its `lookup` returns visibility-wrapped text, keep its public
signature stable if possible.

ALSO (same lane, one small bonus with its own commit): the legacy bridge
copy at `compute/core/src/import/parse_output_to_snapshot/sheet_lowering.rs`
(~:97-98, `formula: cell.formula.clone()` into `snapshot_types::CellData`
whose own doc comment says "Legacy... When `identity_formula` is present, it
takes precedence" — `snapshot-types/src/init.rs:335-338`). Investigate: at
that lowering site, is `identity_formula` ALWAYS populated when
`cell.formula` is? If yes, gate the clone: only populate the legacy field
when `identity_formula` is None. If the population invariant is unclear,
STOP on that sub-change and report (skip it, finish the Arc work) — do not
guess.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Scheduler tests | `cargo test -p compute-core --lib scheduler` | all pass |
| Formula-text/readback tests | `cargo test -p compute-core --lib formula_text` and `--lib test_deferred_xlsx_import` | all pass |
| Structural-edit regeneration | `cargo test -p compute-core --lib edit` | all pass |
| Full engine | `cargo test -p compute-core --lib` | zero failures (009 landed) or only documented baselines |

## Scope

**In scope**: `compute/core/src/scheduler/mod.rs`, `scheduler/init.rs`,
`scheduler/edit.rs`, `compute/core/src/formula_text.rs`, the callers the
type change breaks (compiler-enumerated), and the one gated-clone site in
`sheet_lowering.rs`.

**Out of scope**: the Yrs document's `"f"`/`"fm"` keys (collab source of
truth + import fidelity); `domain_types::CellData.formula` /
`.cell_formula.text` (I/O fidelity, plan 010 owns that file's layout);
merging the two maps into one (the doc comment forbids it — respect it).

## Git workflow

Same worktree/branch as plan 010 (`/Users/vish/Repos/analyst/mog-opt-e`,
`opt/celldata-diet`), separate commits, prefix `sapiex-patches:`, no push.

## Steps

1. Type change + seed/regeneration sharing; compiler-driven caller fixes.
   **Verify**: check + scheduler/edit/formula_text suites.
2. Add a test: after init from a fixture with formulas, assert
   `Arc::ptr_eq` between the two maps' values for one cell (prove sharing);
   after a structural edit triggering regeneration, assert sharing again.
3. The gated legacy clone in `sheet_lowering.rs` (or STOP that sub-step per
   above). **Verify**: full compute-core lib.

## Done criteria

- [ ] Both maps hold `Arc<str>`; seed + regeneration share allocations
      (ptr_eq test proves it)
- [ ] `FormulaTextProvider` behavior unchanged (its tests pass untouched)
- [ ] Full compute-core lib at expected outcome
- [ ] Legacy-clone sub-step done or explicitly STOP-reported
- [ ] README row updated

## STOP conditions

- A consumer requires `&mut String` in-place mutation of map values —
  report the site.
- The `identity_formula` population invariant can't be established from
  source — skip sub-step 3, report.
- Persistent failure after two attempts.
