# Plan 016: Parse allocation diet — CPU and transient-memory churn

> **Executor instructions**: Follow steps in order; each has its own
> verification. On any STOP condition, stop and report. Update row 016 in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done (you may write
> to that one file outside your worktree).
>
> **Drift check**: base `984efd41`; `git diff --stat 984efd41..HEAD -- file-io/xlsx/parser/src/` empty in your worktree before starting.

## Status
P1 / Effort M-L total (steps are S each) / Risk LOW-MED / Planned at `984efd41`.

## Why: measured evidence

Native full import of the 19.4MB / 1.4M-cell repro: 17s wall, ~13GB RSS peak
of which ~10GB is TRANSIENT allocation churn (measured: dropping the engine
returns RSS only to 10.4GB; a second import adds just 2.9GB — the live set).
A CPU sample during import shows the top active frames are allocator
machinery (`_xzm_xzone_malloc_tiny` 2969 samples, freelist 1761, `_xzm_free`
1628, memmove 656 — more than any engine code). Every step below removes
per-cell allocations from the parse hot path; wins apply to BOTH the server
import and the browser (landing-sheet parse + per-sheet materialization).

## Verification commands (used per step)
- `cargo check --workspace --locked` → exit 0
- `cargo test -p xlsx-parser` → 0 failures (full suite ~3,572 tests)
- `cargo test -p compute-core --lib test_deferred_xlsx_import` → all pass
- End-of-plan A/B: `node --expose-gc <scratch>/mog-ab-measure.mjs` style
  timing is run by the ADVISOR post-merge; your gate is tests + check.

## Scope
**In**: `file-io/xlsx/parser/src/` only. **Out**: `compute/core`,
`domain-types` layout, the wasm/napi builds, `StrInternPool` API (extend
usage, don't redesign).

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-i`, branch `opt/parse-alloc-diet`
(created for you), commit per step, prefix `sapiex-patches:`, no push.

## Steps (priority order — commit each; if one STOPs, continue to the next)

### 1. Numeric-provenance check without `format!` (every numeric cell)
`output/to_parse_output/cells.rs:257-262,388-409`: when
`compact_numeric_provenance` (default true), each numeric cell runs
`format!("{n:.0}")` or `format!("{n}")` + `trim_end_matches` + String
compare just to decide whether original_value can be dropped. Replace with
an allocation-free check: write the float's canonical form into a stack
buffer (`ryu` if already a dep — check Cargo.toml; else `write!` into
`arrayvec`-style fixed buffer or `core::fmt` into a `[u8; 24]` wrapper) and
compare bytes. Behavior identical (same strings compared).
**Verify**: parser suite; specifically the `original_numeric_value` tests in
`write/from_parse_output/sheet_cells.rs` + `to_parse_output` tests.

### 2. Shared-formula reference expansion cache (every drag-filled cell)
`domain/cells/helpers/formula_refs.rs:1-30,95-197` (called from
`domain/cells/full_convert.rs:342-357`): each reference cell of a shared
formula re-scans the master formula's full text and allocates a ~10-byte
String per A1 match. Tokenize each MASTER formula once into a cached
segment list (literal spans + reference spans with parsed row/col), keyed
by shared-formula index (the `si` attribute — the master lookup already
exists there); expanding a reference cell then walks the cached segments
and writes one output String (single allocation, correct capacity).
**Verify**: parser suite; shared-formula tests
(`grep -rln "shared.*formula" file-io/xlsx/parser/src --include=*test*` and
the `structural_shared_formulas` compute-core tests via the deferred suite).

### 3. Intern at first SST resolution (every string cell)
`domain/cells/helpers/scan.rs:361` byte-copies every shared-string
occurrence into a per-sheet buffer, then `full_convert.rs`
(`bytes_to_string`/`decode_xstring_to_string`) allocates an owned String
per cell, and only later does `StrInternPool` dedupe (memory, not alloc
count: 100K rows referencing one SST entry still pay 100K copies+allocs).
Build `Vec<Arc<str>>` alongside the SST once per workbook; make the
cell-value carrier hold the `Arc<str>` (or the SST index resolved late) so
per-cell copies disappear. NOTE `FullCellData.value: Option<String>` has
many consumers — the surgical variant: keep `value: Option<String>` for
non-SST cells but add `sst_resolved: Option<Arc<str>>` used preferentially
by `to_parse_output` (which already has the interner — an SST-provided
`Arc<str>` bypasses `pool.intern` via a pool pre-seed or direct use).
Choose the least-invasive shape that removes the per-cell copy; explain
your choice in the commit message.
**Verify**: parser suite; `Arc::ptr_eq` interning tests still pass;
compute-core deferred suite.

### 4. Skip redundant UTF-8 validation (every string/formula cell)
`domain/cells/full_convert.rs:48-53`: `std::str::from_utf8().expect()` per
cell where the comment states the archive boundary already validated.
AUDIT the invariant first: find the archive-boundary validation (zip
inflate → where bytes are first checked); if any path (recovery/lenient
parse) can deliver unvalidated bytes, STOP this step and report. If
airtight, switch to `debug_assert!(str::from_utf8(..).is_ok())` +
`unsafe { from_utf8_unchecked }` with a SAFETY comment naming the boundary.
**Verify**: parser suite + one new test feeding the public parse API
invalid UTF-8 inside a cell value, asserting a clean error (not UB/panic)
— it must be rejected at the boundary you documented.

### 5. Small wins bundle
(a) `output/to_parse_output/sheet/mod.rs:81,93-100`: per-cell
`projection_roles.get(&(row,col))` — add `roles.is_empty()` fast path
hoisted out of the loop (roles are empty for >99% of sheets).
(b) `pipeline/lazy/cells.rs:52`, `pipeline/lazy/streaming_sheet.rs:135`,
`pipeline/parallel.rs:322`: `shared_string_refs: Vec<&str>` rebuilt per
sheet/per rayon task — build once at workbook init, pass `&[&str]`.
(c) `pipeline/lazy/cells.rs:35-39,45-92`: cell-count heuristic
(`size/50`) can undershoot → full re-parse; use the exact
`count_worksheet_cell_elements` (already exists in the call graph).
**Verify**: parser suite after each sub-item.

## Done criteria
- [ ] All committed steps: parser suite 0 failures; compute-core deferred suite green; cargo check clean
- [ ] `cargo fmt --all -- --check`: no NEW drift (bridge macro drift is pre-existing upstream, leave it)
- [ ] Commit messages state, per step, what allocation per WHAT unit was removed
- [ ] README row 016 updated listing which steps landed / STOPped

## STOP conditions
- Step 4's invariant can't be proven airtight — skip step, report.
- Step 3's carrier change fans out beyond the parser crate — STOP that step
  (it becomes a designed follow-up), finish the rest.
- Any step: persistent failure after two attempts — skip, note, continue.
