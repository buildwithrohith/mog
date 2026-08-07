# Plan 023: Remove persisted idToPos via schema-v20 compacting cutover

> The FULL spec is /Users/vish/Repos/analyst/mog/plans/008-DESIGN-OUTPUT.md
> section 6 — read it first and treat it as binding. This plan adds execution
> order, boundaries, and verification.
> **Drift check**: base `c7f9f39c`;
> `git diff --stat c7f9f39c..HEAD -- compute/core/src/storage/ compute/core/crates/compute-document/` empty in YOUR worktree.

## Status
P1 / L / Risk HIGH (CRDT schema migration). Planned at `c7f9f39c`.

## Why
Every hydrated cell writes TWO persisted Y.Map entries (posToId + idToPos).
The design measured this as 40-50% of per-cell Yrs cost. posToId is already
the CRDT authority everywhere that matters (snapshots.rs:218-280 iterates it
precisely because idToPos may hold losing IDs after concurrent writes).

## Execution order
1. **Transient reverse index first.** Add a per-sheet transient reverse map
   (built once from posToId on sheet rebuild) and port the six consumer sites
   listed in design §6 (storage_methods.rs:88-150, values.rs:108-139,
   sync_pipeline/repair.rs, comments/cleanup.rs:12-33, sheet/crud.rs:174-216,
   compute-document/src/schema.rs) to it. Commit; full suite green BEFORE any
   schema change.
2. **Schema 19→20**: v20 writers create only posToId. Bump the schema
   constant, update construction/assertions in compute-document/schema.rs.
3. **Compacting load**: a v20 binary loading a v19 doc rebuilds a FRESH v20
   Doc from authoritative posToId state and persists/attaches from that
   compact state (design §6 point 2 — deleting the Y.Map in place is NOT
   acceptable: tombstones survive and the memory win is unproven).
4. **Refusal path**: verify the existing max-supported-schema check makes
   older binaries refuse v20 (find it; add a test loading a v20 doc under a
   simulated lower max — model on existing schema-version tests).

## Verify
Per step: `cargo test -p compute-core --lib` 0 failures;
`cargo test -p compute-document` 0 failures. New tests: (a) v19 doc loads,
compacts, round-trips values+identities byte-stable on second save; (b) v20
doc contains NO idToPos map entry for any hydrated sheet (assert on the doc
schema, not just absence of reads); (c) undo/observer recovery and sheet-copy
still resolve positions (the ported consumers' existing tests are the net).
Measure: encode_full_state byte size for a fixture sheet before/after — commit
message must state the measured reduction.

## Scope
**In**: compute/core/src/storage/**, compute/core/crates/compute-document/**,
tests. **Out**: mirror/ (plan 024 owns it), kernel TS, wasm bridge surface,
provider/sync protocol beyond what §6 point 3 requires — if provider baseline
replacement turns out to need protocol changes outside this repo, STOP and
report (design §6 point 3).

## Git workflow
Worktree /Users/vish/Repos/analyst/mog-opt-p, branch opt/schema-v20-cutover,
commit per step, prefix `sapiex-patches:`, no push. Only external writable
file: plans/README.md (row 023). Never run git in the main repo.

## STOP conditions
- Consumer port (step 1) changes observable undo/redo behavior — report.
- Compacting load can't preserve identity stability for concurrent-edit
  fixtures — report with the failing fixture, do not weaken the test.
- Provider baseline coordination requires cross-repo changes — report.
- Two failed attempts at any step.
