# Plan 019: Fast-path parse robustness — stop losing cells silently

> Follow steps in order; each verified. Update row 019 in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
> **Drift check**: base `984efd41`; `git diff --stat 984efd41..HEAD -- file-io/xlsx/parser/src/domain/cells/` empty in your worktree. NOTE plan
> 016's lane also works in `file-io/xlsx/parser` but in DIFFERENT files
> (to_parse_output/, helpers/formula_refs.rs, full_convert.rs, pipeline/);
> your files are `domain/cells/parsing/fast.rs`, `domain/cells/helpers/scan.rs`,
> and the diagnostics plumbing. Do not touch 016's files.

## Status
P0 (silent data loss) / M / Risk MED (hottest loop — no throughput regression allowed) / Planned at `984efd41`.

## Why (verified evidence)

Production import uses ONLY the fast path (`xlsx_api::parse` →
`parse_worksheet_fast_with_extras` → `fast.rs` → `scan_cell`); the
diagnostics-equipped recovery path (`domain/cells/recovery.rs`,
`parsing/recovery_path.rs`) is UNREACHABLE from it. Three silent failures:

1. **Worksheet truncation** — `fast.rs:66-77`: `scan_cell` → `None` →
   `break`: every remaining cell in the sheet silently dropped. `scan_cell`
   returns None at `scan.rs:57` (bad `<c` tag) and `scan.rs:314-318`
   (missing closing tag). The buffer-retry loop
   (`pipeline/full_parse/implementation.rs:~1316-1354`) only detects
   `cell_count == buffer_size`, so a malformed-XML abort is never retried
   or reported.
2. **Cell misplacement** — `scan.rs:144-149`: unparseable `r=` attribute
   leaves row/col at the PREVIOUS cell's values: value lands on wrong cell.
3. **SST index corruption** — `scan.rs:260-271`: out-of-range shared-string
   index renders the raw index digits as literal cell text.

The diagnostic taxonomy already exists (`domain-types/src/diagnostics.rs`:
`InvalidCellReference`, `InvalidSharedStringIndex`, etc.) and the engine
already logs `error_count` (`construction/xlsx.rs:108-114` — the c648k
fixture's `error_count=1` came from the document-level context, which
CANNOT see these three).

## Steps

### 1. Thread a lightweight diagnostics sink into the fast path
Add a minimal collector param (e.g. `&mut FastParseDiagnostics { counts per
code + first-N samples with byte offset/row }`) through
`parse_worksheet_fast_with_extras` → `parse_worksheet_core` → `scan_cell`.
It must be allocation-free on the happy path (plain counters; samples via
fixed-size array). Wire its totals into the existing `ParseDiagnostics` /
`ImportReport` that `construction/xlsx.rs` already logs.

### 2. Truncation → resync + report
On `scan_cell` None: record the diagnostic, then RESYNC — scan forward for
the next `<c ` or `</row>` boundary and continue — instead of `break`. If
resync fails (no further boundary), break but with the diagnostic recorded.

### 3. Misplacement + SST fallback → report
Bad `r=`: count `InvalidCellReference` and SKIP the cell (a skipped cell
with a diagnostic beats a misplaced value; matches the recovery path's
choice). Out-of-range SST: keep a visible placeholder — use the recovery
path's `"#REF!"` convention — and count `InvalidSharedStringIndex`.

### 4. Throughput guard
Add/verify a benchmark-style test or timing assertion is NOT feasible in CI;
instead prove no hot-loop regression structurally: the collector is passed
by `&mut`, no per-cell allocation added (assert by review + a test that a
clean parse yields zero diagnostics). State in the commit message what the
happy-path cost is (should be: nothing but an occasional branch).

### 5. Tests
New tests with hand-built malformed XML: (a) one broken cell mid-sheet →
later cells still parsed + diagnostic counted; (b) bad `r=` → cell skipped
+ counted, neighbors correct; (c) SST index 99 of 3 → `#REF!` + counted;
(d) clean file → zero diagnostics, identical output to before (fixture
compare). Model on existing parser tests.

## Scope
**In**: `domain/cells/parsing/fast.rs`, `domain/cells/helpers/scan.rs`,
diagnostics plumbing files, tests. **Out**: recovery_path.rs (leave), plan
016's files, any perf rework beyond the diagnostics.

## Verify (each step)
`cargo check --workspace --locked`; `cargo test -p xlsx-parser` 0 failures;
`cargo test -p compute-core --lib test_deferred_xlsx_import` all pass.

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-l`, branch `opt/parse-robustness`,
prefix `sapiex-patches:`, no push.

## Done criteria
- [ ] The three failure modes each produce counted diagnostics; truncation resyncs
- [ ] Clean-parse output byte-identical (fixture test)
- [ ] Suites green; README row updated

## STOP conditions
- Resync cannot be made safe (risk of re-reading a cell twice) after two
  attempts — ship diagnostics-only (no resync) and report.
- Any existing parser test changes behavior — report, don't adapt tests.
