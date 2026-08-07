# Plan 010: CellData diet — box the formula/fidelity payloads out of every cell

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: base is `80f56679` on `sapiex-patches`.
> `git diff --stat 80f56679..HEAD -- domain-types/src/parse_output.rs` in
> YOUR worktree must be empty before you start.

## Status

- **Priority**: P1 (largest measured static cost)
- **Effort**: M
- **Risk**: LOW-MED (mechanical but wide; serde compat is the trap)
- **Depends on**: none (009 touches different files)
- **Category**: perf/memory
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

`type_size_report` (added in plan 007; `compute/core/tests/type_size_report.rs`)
MEASURES `domain_types::parse_output::CellData` at **600 bytes**, and the
retained parse output holds one per cell: **844.75MB of fixed overhead at the
1,407,921-cell repro**, against a 4GiB wasm32 ceiling. The layout audit
(verified in source) shows most of that weight serves only formula cells or
XLSX round-trip fidelity, yet is paid inline by EVERY cell:

- `formula_cache_provenance: FormulaCacheProvenance` — 136 bytes EMBEDDED
  unconditionally (`parse_output.rs:1626`, struct at `:100-130`: four
  `Option<String>` fingerprints = 96B + two `Option<u64>` + flags). The only
  non-default constructor short-circuits to `default()` for non-formula
  cells (`file-io/xlsx/parser/src/output/to_parse_output/cells.rs:312-320`).
- `cell_formula: Option<ooxml_types::worksheet::CellFormula>` — the inline
  `CellFormula` (`file-io/ooxml/types/src/worksheet/cell_types.rs:136-163`)
  carries `text: String` + ~11 fields ≈ 110B inline, populated only for
  shared/array/data-table formulas.
- `rich_string: Option<RichSharedString>` (`parse_output.rs:1255-1271`:
  String + two Vecs + two Options) — rare.
- Five more `Option<String>`s (`formula`, `array_ref`, `date_lexical_value`,
  `original_value`, + provenance internals) at 24B each, mostly None.

The CSV producer (`file-io/csv-parser/src/parse_output_assembly.rs:166-174`)
populates only 6 of 18 fields — 12 fields are pure default for every
CSV-origin cell.

Blast radius is BOUNDED: the compute engine's hot path never reads this
struct — it lowers to `snapshot_types::CellData` (7 fields,
`compute/core/crates/types/snapshot-types/src/init.rs:323-347`). Consumers
of the wide struct are the XLSX/CSV parsers and writers, the
parse-output→snapshot lowering, hydration, and export — I/O boundary only.

## Target layout

Split `CellData` into a lean base + one boxed sidecar:

```rust
pub struct CellData {
    pub row: u32,
    pub col: u32,
    pub value: CellValue,
    pub style_id: Option<u32>,
    pub projection_role: ImportedCellProjectionRole,
    pub phonetic: bool,
    /// Formula + import-fidelity payload; None for plain value cells.
    pub extras: Option<Box<CellDataExtras>>,
}

pub struct CellDataExtras {
    pub rich_string: Option<RichSharedString>,
    pub formula: Option<String>,
    pub array_ref: Option<String>,
    pub cell_formula: Option<ooxml_types::worksheet::CellFormula>,
    pub cell_metadata_index: Option<u32>,
    pub formula_result_type: Option<u8>,
    pub has_empty_cached_value: bool,
    pub formula_cache_provenance: FormulaCacheProvenance,
    pub vm: Option<u32>,
    pub date_lexical_value: Option<String>,
    pub original_sst_index: Option<u32>,
    pub original_value: Option<String>,
}
```

One sidecar (not two) keeps call sites simple; a value cell pays 8 bytes.
Expected: base CellData well under 120B → roughly **-450 to -550MB** at the
repro's scale for value-heavy workbooks.

**Serde compatibility is REQUIRED**: `ParseOutput` is serialized (Serialize/
Deserialize derives with camelCase). Keep the WIRE FORMAT IDENTICAL by
implementing `#[serde(flatten)]` on an accessor-shaped shim, OR custom
(de)serialize that flattens `extras` into the same top-level camelCase keys
with the same skip/default behavior as today. Write a serde round-trip test
FIRST (Step 1) capturing today's JSON for a formula cell and a plain cell;
that JSON must be byte-identical after the refactor. If today's derives
skip-none fields, match that.

Provide accessor methods on `CellData` (`formula()`, `cell_formula()`,
`formula_cache_provenance()` returning `&FormulaCacheProvenance` with a
`static DEFAULT`, `rich_string()`, etc.) so most read sites change from
field access to method call mechanically. Sites that WRITE fields go through
`extras_mut()` (allocating the box on first write).

## Known access sites (leads, verify by compiler)

- Sole non-default provenance constructor:
  `file-io/xlsx/parser/src/output/to_parse_output/cells.rs:289,312-360`
- Provenance readers (~24 sites): `write/from_parse_output/sheet_cells.rs:206`,
  `export_report.rs:76,111`, `output/to_parse_output/sheet_extents.rs:35,55`,
  `compute/core/src/storage/infra/hydration/{features.rs:115,styles.rs:632+685,sheet/identity.rs:20}`,
  `compute/core/src/storage/properties/cell.rs:289-343`,
  `viewport/functions/active_cell.rs:201`,
  `services/export/cells/materialize.rs:62,243`,
  `versioning/semantic_reader/value_provenance.rs:165-168`,
  `domain-types/src/yrs_schema/cell_properties.rs:91-92`
- NOTE: `snapshot-types/src/properties.rs:99,155` and
  `domain-types/src/domain/formatting.rs:66,141` declare their OWN
  `formula_cache_provenance` fields on DIFFERENT structs — leave those alone.
- Producers: `to_parse_output/cells.rs:264-309` (XLSX),
  `csv-parser/src/parse_output_assembly.rs:166-174` (CSV)
- Lowering: `compute/core/src/import/parse_output_to_snapshot/sheet_lowering.rs`
- Let `cargo check` enumerate the rest; it is the authoritative list.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Size proof | `cargo test -p compute-core --test type_size_report -- --nocapture` | CellData size printed **< 150 bytes**; update the test's ceiling assert accordingly and add `CellDataExtras` to the table |
| Parser suites | `cargo test -p xlsx-parser -- --test-threads=1` | zero failures if plan 009 landed; else only the two documented baseline failures |
| Roundtrip fidelity | `cargo test -p xlsx-parser --test roundtrip_parse_output` | same as above |
| Engine consumers | `cargo test -p compute-core --lib` | same as above |
| CSV | `cargo test -p csv-parser` (verify package name in `file-io/csv-parser/Cargo.toml`) | all pass |

## Scope

**In scope**: `domain-types/src/parse_output.rs`; every compile-error site
the split creates; `compute/core/tests/type_size_report.rs` (new numbers);
one new serde-compat test file in `domain-types`.

**Out of scope**: `snapshot_types::CellData` (already lean);
`CellValue` (plan 012); any behavior change — this is layout only;
`domain/formatting.rs` and `snapshot-types/properties.rs` provenance fields.

## Git workflow

Worktree `/Users/vish/Repos/analyst/mog-opt-e`, branch `opt/celldata-diet`
(will be created for you), commit prefix `sapiex-patches:`, do not push.

## Steps

### Step 1: Pin the wire format

Write the serde round-trip test capturing CURRENT JSON for (a) a fully
populated formula cell, (b) a plain value cell, (c) a rich-string cell.
Commit it passing against the OLD layout first.

### Step 2: Introduce the split with accessors

New structs + accessor methods + custom/flattened serde. Fix all compile
errors mechanically (reads → accessor, writes → `extras_mut()`). The XLSX
producer should construct `extras` ONCE per cell only when any payload field
is non-default (the existing `is_formula` and metadata checks at
`cells.rs:264-309` tell you when).

**Verify**: `cargo check` exit 0; Step-1 test still passes byte-identical.

### Step 3: Prove the size and run the world

Update `type_size_report` ceilings; run all suites in the table.

## Done criteria

- [ ] `size_of::<CellData>()` printed < 150B; report includes the new table
- [ ] Step-1 serde test passes with byte-identical JSON
- [ ] All suites in the commands table at expected outcomes
- [ ] No behavior change (no test assertions modified other than
      type_size_report ceilings)
- [ ] README row updated

## STOP conditions

- The serde derives turn out to produce field-order-dependent output that
  flatten cannot reproduce byte-identical — report the delta; a
  semantically-equal (key-set-equal) guarantee may be acceptable but the
  ADVISOR decides, not you.
- Any consumer mutates provenance in place on a cell that has no extras yet
  in a hot loop (allocation storm risk) — report the site.
- Persistent failure after two attempts.

## Maintenance notes

- Reviewers: check the XLSX producer doesn't allocate `extras` for plain
  cells (the whole point); a `debug_assert` or test counting extras on a
  values-only fixture is welcome.
- The lowering bridge (`sheet_lowering.rs`) reads `formula`/`array_ref` —
  after this plan they come via accessors; plan 011 then touches the same
  file to drop a legacy clone. Land 010 before 011 (same lane).
