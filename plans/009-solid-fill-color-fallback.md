# Plan 009: Solid-fill color fallback — fix the two baseline fill-color failures

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git -C /Users/vish/Repos/analyst/mog rev-parse --short HEAD`
> must be `80f56679` or a descendant on `sapiex-patches`;
> `git diff --stat 80f56679..HEAD -- file-io/xlsx/parser/src/write/` must be
> empty (no one else has touched the writer). On drift, compare excerpts
> before proceeding.

## Status

- **Priority**: P1 (correctness: exported workbooks lose solid fill colors)
- **Effort**: S
- **Risk**: LOW-MED (style-table dedup identity)
- **Depends on**: none
- **Category**: correctness
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

Two tests have been failing since before this optimization effort began (they
are the ONLY failures in the whole workspace, documented in this README as
known-baseline):

1. `compute-core --lib test_xlsx_col_style_ranges::direct_cell_row_col_and_styled_blank_formats_export_through_xlsx`
2. `xlsx-parser --test roundtrip_parse_output styles::roundtrip_fill_formatting`

Both bottom out in the SAME line range: when a `DocumentFormat.fill` has
`pattern_type: Some("solid")` and its color authored in
`pattern_foreground_color` (with `background_color: None`), the OOXML
lowering silently discards the color. Real-world impact: any workbook whose
solid fill reached the engine through the pattern-foreground field exports
with the fill color stripped. Fixing this takes the whole workspace to
zero failing tests, which every future lane depends on for clean signal.

## Current state

Verified at `80f56679`.

`file-io/xlsx/parser/src/write/from_parse_output/styles.rs` (~:365-377), in
`convert_fill`:
```rust
    match pattern_type {
        // An explicit no-fill marker must win over any stale color fields.
        PatternType::None => FillDef::None,
        // DocumentFormat uses `backgroundColor` for a cell's visible solid
        // color. OOXML stores that same color in patternFill/fgColor.
        PatternType::Solid => match background_color {
            Some(fg_color) => FillDef::Solid { fg_color },
            None => FillDef::Pattern {
                pattern_type: Some(PatternType::Solid),
                fg_color: None,               // <-- color dropped here
                bg_color: None,
            },
        },
```
Downstream, `file-io/xlsx/parser/src/domain/styles/write/fills.rs` (~:38-58)
self-closes `<patternFill patternType="solid"/>` when both colors are None,
so nothing survives to the parse side — the loss is write-side only.

Contract that MUST be preserved:
`file-io/xlsx/parser/src/write/from_parse_output/tests/fill_lowering.rs:48-58`
(`explicit_solid_maps_domain_background_to_ooxml_foreground`) pins that when
BOTH `background_color` and `pattern_foreground_color` are set on a solid
fill, `background_color` wins and pattern_foreground is ignored. The fix is
a FALLBACK, not a priority inversion.

Note: `background_color` has a companion `background_color_tint`, and
`pattern_foreground_color` has `pattern_foreground_color_tint` — read how
the Some(fg_color) arm carries tint (look at how `lowered_fill` tests assert
tint) and carry the matching tint through the fallback.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| The two target tests | `cargo test -p compute-core --lib direct_cell_row_col_and_styled_blank_formats_export_through_xlsx` and `cargo test -p xlsx-parser --test roundtrip_parse_output roundtrip_fill_formatting` | both PASS (currently both FAIL) |
| Fill-lowering contract tests | `cargo test -p xlsx-parser fill_lowering` | all pass |
| Full parser + compute-core | `cargo test -p xlsx-parser -- --test-threads=1` and `cargo test -p compute-core --lib` | ZERO failures (the baseline exceptions are retired by this plan) |
| Format | `cargo fmt --all -- --check` | only the pre-existing `infra/rust-bridge/bridge-wasm/macros/src/expand/method.rs` drift remains |

## Scope

**In scope**:
- `file-io/xlsx/parser/src/write/from_parse_output/styles.rs` (`convert_fill`
  Solid arm only)
- `file-io/xlsx/parser/src/write/from_parse_output/tests/fill_lowering.rs`
  (new fallback tests)

**Out of scope**:
- `file-io/xlsx/parser/src/write/from_parse_output/sheet_cells.rs:90-103`
  (silently drops an AuthoredStyleRun when the style remapper can't resolve
  it — a SEPARATE latent issue, documented as a held follow-up; do not fix
  here)
- The parse side (`style_resolver`) — nothing is wrong there
- The two failing tests' assertions — do NOT weaken them; they must pass as
  written

## Git workflow

Branch `opt/fill-color-fallback` in a fresh worktree
(`git -C /Users/vish/Repos/analyst/mog worktree add ../mog-opt-d -b opt/fill-color-fallback`),
commit prefix `sapiex-patches:`, do not push.

## Steps

### Step 1: The fallback

In the `PatternType::Solid` arm, when `background_color` is None, fall back
to `pattern_foreground_color` (with its tint) before emitting the colorless
pattern:
```rust
PatternType::Solid => match background_color.or(pattern_foreground_color_as_solid) { ... }
```
(shape it to the actual variable names in the function — the function
destructures the FillFormat at the top; read ~:330-365 first). Priority
stays: `background_color` first, `pattern_foreground_color` only as
fallback, colorless pattern only when BOTH are absent.

**Verify**: the two target tests pass; `fill_lowering` suite still passes
(especially `explicit_solid_maps_domain_background_to_ooxml_foreground`).

### Step 2: Pin the new behavior

Add to `fill_lowering.rs`:
- `solid_with_only_pattern_foreground_falls_back_to_it` (fallback works,
  tint carried)
- `solid_with_neither_color_still_emits_colorless_pattern` (the both-absent
  case unchanged — dedup identity for existing colorless solids preserved)

**Verify**: full parser suite zero failures; full compute-core lib zero
failures.

### Step 3: Retire the baseline exceptions

Update `/Users/vish/Repos/analyst/mog/plans/README.md`: move both entries in
"Known baseline test failures" to a "FIXED by plan 009" note, and update
your row.

## Done criteria

- [ ] Both formerly-failing tests pass AS WRITTEN (assertions untouched)
- [ ] `fill_lowering` contract tests pass (background_color priority intact)
- [ ] `cargo test -p xlsx-parser -- --test-threads=1` → 0 failures
- [ ] `cargo test -p compute-core --lib` → 0 failures
- [ ] `cargo fmt --all -- --check` → only the pre-existing bridge-macro drift
- [ ] No files outside Scope modified; README updated

## STOP conditions

- Making the fallback pass the two tests BREAKS
  `explicit_solid_maps_domain_background_to_ooxml_foreground` — report; do
  not change that test.
- The fix changes fill dedup such that OTHER style tests fail (style-table
  identity shifted) — report which, do not chase.
- Persistent failure after two attempts.

## Maintenance notes

- Reviewers: the risk is the both-set priority case and fill dedup identity;
  Step 2's second test guards the colorless case staying byte-identical.
- Held follow-up (README): the silent AuthoredStyleRun drop at
  `sheet_cells.rs:90-103` should at minimum log; separate small plan.
