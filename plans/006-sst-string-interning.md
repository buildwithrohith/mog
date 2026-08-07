# Plan 006: Intern duplicate cell strings at parse time (SST dedup restored)

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- file-io/xlsx/parser/src/`
> On any change since 60b6ee3e, compare excerpts before proceeding.

## Status

- **Priority**: P1
- **Effort**: S-M
- **Risk**: LOW
- **Depends on**: none (parser crate; independent of plans 001-005)
- **Category**: perf
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

XLSX files dedupe repeated text via the Shared String Table precisely because
spreadsheets repeat text massively (headers, categories, status labels). This
parser resolves each SST reference into an owned `String` per cell
(`full_convert.rs`), then wraps each in a **fresh** `Arc<str>` per cell
(`cells.rs`) — the `_shared_strings` parameter at the value-construction
layer is literally underscore-unused. `CellValue::Text(Arc<str>)`'s own doc
comment says the `Arc` choice exists so millions of text cells can share
allocations; nothing ever shares them. Two independent audit passes (parser
and live-engine) converged on this. An intern pool at parse time collapses
every repeated string — SST-referenced or inline — to one allocation, cutting
both bytes and malloc pressure (worst on wasm32) for every text-heavy
workbook, on every open, in both the browser and native builds.

## Current state

Verified at `60b6ee3e`.

`file-io/xlsx/parser/src/output/to_parse_output/cells.rs:398-429`:
```rust
pub(super) fn resolve_cell_value(cell: &FullCellData, _shared_strings: &[String]) -> CellValue {
    ...
        CELL_TYPE_STRING => cell
            .value
            .as_ref()
            .map(|v| CellValue::Text(Arc::from(v.as_str())))     // fresh Arc per cell
            .unwrap_or(CellValue::Text(Arc::from(""))),
        CELL_TYPE_DATE => ... CellValue::Text(Arc::from(v.as_str())) ...,
    // number-fallback branch also does Arc::from(v.as_str())
```
`resolve_formula_cached_value` in the same file has more `Arc::from` sites
(auditor cited ~`:415-468`; grep `Arc::from` in this file for the full list).

`file-io/xlsx/parser/src/domain/cells/full_convert.rs:235-243` — the SST
index is resolved to an owned clone much earlier:
```rust
        let resolved = value_str.parse::<usize>().ok()
            .and_then(|idx| shared_strings.get(idx).cloned());   // String clone per cell
        if let Some(s) = resolved { cells[cell_idx].value = Some(s); }
```
(Leave this stage alone — `FullCellData.value: Option<String>` feeds many
consumers; the interning point is where `CellValue` is constructed.)

`CellValue::Text` type:
`compute/core/crates/types/value-types/src/cell_value/mod.rs` (~:1-130) —
`Text(Arc<str>)`, doc comment promises O(1) clone sharing.

Conventions: this crate is plain Rust, `FxHashMap` is used across the
workspace (`rustc-hash`); check `file-io/xlsx/parser/Cargo.toml` for whether
`rustc-hash` is already a dependency — if not, use `std::collections::HashMap`
(do NOT add a new dependency for this).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Parser tests | `cargo test -p xlsx-parser` (find exact package name via `grep -m1 '^name' file-io/xlsx/parser/Cargo.toml`) | all pass |
| Engine tests (consumers) | `cargo test -p compute-core --lib test_deferred_xlsx_import` | all pass |

## Scope

**In scope**:
- `file-io/xlsx/parser/src/output/to_parse_output/cells.rs`
- The one struct/context type threaded through `to_parse_output` conversion
  that can carry the pool (find the conversion driver:
  `grep -rn "resolve_cell_value" file-io/xlsx/parser/src/ | grep -v "fn resolve_cell_value"`
  shows the callers; the pool lives for one workbook conversion pass)
- Parser test file(s) adjacent to existing tests in that crate

**Out of scope**:
- `full_convert.rs` / `FullCellData.value: Option<String>` representation
- The live-engine intern pool for EDIT-time strings (held follow-up — this
  plan is parse-time only)
- `compute/core` (no changes; it consumes `CellValue` unchanged)

## Git workflow

Branch `opt/parse-intern-observability` (this worktree), commit prefix
`sapiex-patches:`, do not push.

## Steps

### Step 1: Add the intern pool

In `to_parse_output` (same module as `cells.rs`), add:

```rust
pub(super) struct StrInternPool { map: HashMap<Arc<str>, ()> /* or FxHashMap */ }
impl StrInternPool {
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        // get_key_value hit → Arc::clone; miss → Arc::from(s) inserted then cloned
    }
}
```
(Key by `Arc<str>` with `Borrow<str>` lookup — `HashMap<Arc<str>, ()>` +
`map.get_key_value(s)` avoids double allocation on hit. An
`HashSet<Arc<str>>` with `get_or_insert_with` is not stable; use the map
pattern.)

**Verify**: `cargo check --workspace --locked` → exit 0 (pool unused yet is
fine; silence dead-code with the call in Step 2 or `#[allow]` temporarily).

### Step 2: Thread the pool through cell-value resolution

Replace the unused `_shared_strings: &[String]` parameter of
`resolve_cell_value` (and `resolve_formula_cached_value` if it also
constructs `Text`) with `pool: &mut StrInternPool`, and replace every
`CellValue::Text(Arc::from(v.as_str()))` in this file with
`CellValue::Text(pool.intern(v))`. Construct one pool per conversion pass at
the caller (one per workbook parse; per-sheet is acceptable if the caller
structure makes workbook-scope awkward — note which you chose in the commit
message). Update all call sites (compiler-driven).

**Verify**: `cargo check --workspace --locked` → exit 0; parser tests all
pass.

### Step 3: Prove sharing

New parser test: build a minimal in-memory sheet (model on the crate's
existing conversion tests — find them via
`grep -rn "resolve_cell_value\|to_parse_output" file-io/xlsx/parser/src --include=*test*` or
`#[cfg(test)]` modules in `to_parse_output/`) with two string cells holding
identical content; convert; assert the two `CellValue::Text` arcs satisfy
`Arc::ptr_eq`. Add a distinct-content pair asserting NOT `ptr_eq`.

**Verify**: parser tests → all pass including the two new ones;
`cargo test -p compute-core --lib test_deferred_xlsx_import` → all pass
(consumer regression).

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] Parser + compute-core focused tests exit 0
- [ ] `grep -c "Arc::from" file-io/xlsx/parser/src/output/to_parse_output/cells.rs` → only the empty-string fallback remains (or 0 if you interned "" too)
- [ ] `Arc::ptr_eq` test proves dedup
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- `resolve_cell_value`'s callers cannot thread `&mut` state (e.g. parallel
  iterator over cells) — report the parallelism structure; a per-shard pool
  merged at the end is the fallback, but propose before building it.
- Parser package tests don't exist / can't run standalone — report the test
  layout you found.
- Persistent failure after two attempts.

## Maintenance notes

- Held follow-up (README): a workbook-lifetime intern pool in the live
  engine for edit-time strings, and lookup-index lowercase copies — this plan
  only covers parse-time construction, which is where the 1.4M-cell open
  cost lives.
- Reviewers: confirm the pool is dropped at end of conversion (it must not
  outlive the parse; retained interning would pin every string ever seen).
