# Plan 005: Wire the dead sheet-switch cleanup APIs (palettes + viewport state)

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- compute/core/src/storage/engine/viewport/ apps/spreadsheet/src/systems/grid-editing/subscriptions/sheet-switch-coordination.ts kernel/src/document/document-lifecycle-system.ts`
> On any change to these since 60b6ee3e, compare excerpts before proceeding.

## Status

- **Priority**: P2
- **Effort**: S-M
- **Risk**: LOW-MED
- **Depends on**: none (independent of plans 001-004; different files)
- **Category**: perf
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

Per-sheet viewport state accumulates for every sheet a user visits and is
never released: format palettes are documented "cleared on sheet switch" but
no code path clears them per-sheet, and the purpose-built cleanup APIs
(`reset_sheet_viewports`/`reset_viewport_state` Rust-side,
`clearPerViewportState`/`resetViewportState` bridge-side) have ZERO
production callers — the sheet-switch handler calls none of them. Separately,
two deferred-materialization sites refresh EVERY registered coordinator
instead of only the affected sheet's. A 28-tab browse retains 28 sheets of
palette + viewport state until document dispose. This plan wires the
existing primitives into the actual switch path.

## Current state

Verified at `60b6ee3e`:

`compute/core/src/storage/engine/viewport/service.rs:46-77`:
```rust
pub(crate) struct ViewportService {
    registered_viewports: RefCell<FxHashMap<String, ViewportRegistration>>,
    /// Per-sheet format palettes for binary viewport transfer.
    /// Append-only within a sheet; cleared on sheet switch.   // <-- FALSE today
    format_palettes: RefCell<FxHashMap<SheetId, format_palette::FormatPalette>>,
}
// clear() and clear_all_palettes() wipe the WHOLE map; no per-sheet removal exists.
```

`compute/core/src/storage/engine/viewport/functions/registry_ops.rs:100-119`
— two functions with IDENTICAL bodies, neither touches palettes:
```rust
pub(...) fn reset_sheet_viewports(viewport: &ViewportService, sheet_id: &SheetId) -> ... {
    viewport.registered_viewports_mut().retain(|_, reg| reg.sheet_id != *sheet_id);
    Ok(MutationResult::empty())
}
pub(...) fn reset_viewport_state(...) -> ... { /* same body */ }
```

`apps/spreadsheet/src/systems/grid-editing/subscriptions/sheet-switch-coordination.ts:218+`
— `setupSheetSwitchCoordination` handles save/restore of view state, editor
commit, renderer/chart actor messages; it never calls any bridge cleanup.
Bridge methods exist: `kernel/src/bridges/compute/compute-bridge.gen.ts` has
`resetViewportState` (grep `:205,1442` area) and
`kernel/src/bridges/compute/viewport-fetch-manager.ts:681-710` has
`forceRefreshAllViewports()` (unscoped) and
`forceRefreshSheetViewports(sheetId)` (scoped).

`kernel/src/document/document-lifecycle-system.ts` — two call sites use the
unscoped refresh where a sheet id is in hand: ~`:948`
(`materializeReadOnlySheet` completion) and ~`:1835-1842` (belt-and-suspenders
post-hydration). Grep `forceRefreshAllViewports` to locate exactly; there may
be more sites — only convert ones where the affected sheet is known.

Existing tests to model on / extend:
`kernel/src/bridges/compute/__tests__/viewport-sheet-switch.test.ts` (calls
`clearPerViewportState` directly today).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Rust typecheck | `cargo check --workspace --locked` | exit 0 |
| Rust viewport tests | `cargo test -p compute-core --lib viewport` | all pass |
| TS install (once) | `pnpm install` | exit 0 |
| Kernel typecheck | `pnpm --filter @mog-sdk/kernel typecheck` | exit 0 |
| Kernel tests | `pnpm --filter @mog-sdk/kernel test --runInBand src/bridges/compute/__tests__/viewport-sheet-switch.test.ts` | all pass |
| App typecheck | `pnpm --filter @mog/app-spreadsheet typecheck` | exit 0 |

## Scope

**In scope**:
- `compute/core/src/storage/engine/viewport/service.rs` (add per-sheet
  palette removal; fix the stale doc comment)
- `compute/core/src/storage/engine/viewport/functions/registry_ops.rs`
- `apps/spreadsheet/src/systems/grid-editing/subscriptions/sheet-switch-coordination.ts`
- `kernel/src/document/document-lifecycle-system.ts` (the refresh-scoping
  swap only)
- Tests adjacent to each

**Out of scope**:
- `construction/deferred.rs` (plans 001-004 own it — do not touch)
- Decoration stores (may be intentionally cross-sheet)
- The materialized-sheet LRU (held follow-up)
- Deleting either duplicate reset function's public surface (dedupe their
  BODIES via a shared helper is fine; keep both exported names)

## Git workflow

Branch `opt/viewport-cleanup` (this worktree), commit prefix
`sapiex-patches:`, do not push.

## Steps

### Step 1: Per-sheet palette removal (Rust)

Add `pub fn remove_sheet_palette(&self, sheet_id: &SheetId)` to
`ViewportService` (a `format_palettes.borrow_mut().remove(sheet_id)`).
Call it from BOTH `reset_sheet_viewports` and `reset_viewport_state` in
`registry_ops.rs` (extract a shared body helper). Fix the field's doc
comment to describe reality ("removed when a sheet's viewports are reset").

**Verify**: `cargo check --workspace --locked` → exit 0;
`cargo test -p compute-core --lib viewport` → all pass. Add a small Rust test
asserting: render sheet A (palette entry exists) → `reset_viewport_state(A)`
→ palette map has no entry for A.

### Step 2: Call cleanup on actual sheet switch (TS)

In `setupSheetSwitchCoordination`, at the point where a switch away from
`previousSheetId` is known (read the handler to find where the previous
sheet id is available — the save-view-state step has it), call the bridge's
`resetViewportState(previousSheetId)` fire-and-forget with a `.catch` that
logs a warning (match the file's existing
`console.warn('[SheetSwitchCoordination] ...')` style). Confirm via the
existing test file how the bridge is reached from this layer (the config
object exposes `workbook`; find the path to the compute bridge from it —
STOP if no bridge handle is reachable from this subscription's inputs and
report which object would need to carry it).

**Verify**: `pnpm --filter @mog/app-spreadsheet typecheck` → exit 0.

### Step 3: Scope the two refresh sites (TS)

In `document-lifecycle-system.ts`, at the two sites where
`forceRefreshAllViewports()` is called and the just-materialized/affected
sheet id is in scope, switch to `forceRefreshSheetViewports(sheetId)`.
Leave any site where no specific sheet is identifiable unchanged.

**Verify**: `pnpm --filter @mog-sdk/kernel typecheck` → exit 0;
kernel viewport test file → all pass.

### Step 4: Extend the kernel test

Extend `viewport-sheet-switch.test.ts`: after a simulated switch,
assert the reset path was invoked for the previous sheet (mock/spy at
whatever seam that file already uses), and that a re-visit repopulates
(refresh still works after cleanup).

**Verify**: kernel test command → all pass.

## Done criteria

- [ ] `cargo check --workspace --locked` exit 0; Rust palette test passes
- [ ] Kernel + app typechecks exit 0
- [ ] `viewport-sheet-switch.test.ts` passes with new assertions
- [ ] `grep -n "cleared on sheet switch" compute/core/src/storage/engine/viewport/service.rs` → no stale claim (comment updated)
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- No bridge handle is reachable from `setupSheetSwitchCoordination`'s config.
- `forceRefreshSheetViewports` does not repopulate a just-reset sheet's
  coordinators (regression in re-visit rendering) — report, do not widen.
- Persistent failure after two attempts.

## Maintenance notes

- Reviewers: the risk is a just-switched-TO sheet losing its coordinators if
  reset targets the wrong id — the test in Step 4 must cover switch A→B
  asserting B renders and only A was reset.
- Held follow-up: LRU eviction of materialized ENGINE state (needs kernel
  materialization-tracker coordination; documented in README).
