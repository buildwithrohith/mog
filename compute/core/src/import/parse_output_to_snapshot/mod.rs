//! Converts `ParseOutput` → `WorkbookSnapshot` for compute-core initialization.
//!
//! This is the **new hydration pipeline** replacement for the legacy
//! `ImportSnapshot` → `WorkbookSnapshot` path. `ParseOutput` is position-keyed
//! (no UUIDs, no identity formulas), so this conversion:
//!
//! 1. Generates fresh UUID strings for sheet IDs and cell IDs.
//! 2. Maps `SheetData` cells → `snapshot_types::CellData` (value + formula).
//! 3. Converts `NamedRange` → `NamedRangeDef` (raw expression path).
//! 4. Converts `TableSpec` → `TableDef` (parsing A1 range refs into row/col).
//! 5. Passes through calculation properties (iterative calc settings).
//!
//! Identity formula conversion happens later — the scheduler's
//! `bulk_parse_and_register()` converts A1 formulas to identity formulas
//! using the live `CellMirror` after snapshot loading.
//!
//! # Module layout (W3.0 pre-split)
//!
//! This directory splits the lowering pipeline into per-boundary submodules so
//! W4's fan-out agents don't collide on a single file. Current layout:
//!
//! - [`sheet_lowering`] — sheets + iterative calc
//! - [`name_lowering`] — defined names (boundary 1.1)
//! - [`table_lowering`] — tables (1.10–1.12)
//! - [`pivot_lowering`] — pivot tables (1.17)
//! - [`data_table_lowering`] — data tables (1.5–1.7)
//! - [`validation_lowering`] — data validation (1.2–1.4) [W4 landing pad]
//! - [`hyperlink_lowering`] — hyperlinks (1.13–1.14) [W4 landing pad]
//! - [`sparkline_lowering`] — sparklines (1.8–1.9) [W4 landing pad]
//! - [`merge_lowering`] — merge ranges [W4 landing pad]
//! - [`view_lowering`] — SheetPane (1.15) [W4 landing pad]

use domain_types::ParseOutput;
use rustc_hash::FxHashMap;
use snapshot_types::{SheetSnapshot, WorkbookSnapshot};

use crate::storage::infra::hydration::HydrationIdMap;

// Re-export so external consumers (formula-eval, integration tests) can access
// the allocator type required by `parse_output_to_workbook_snapshot`.
pub use crate::storage::infra::hydration::DefaultIdAllocator;

pub(crate) mod anchor_collection;
pub(crate) mod classifier;
pub mod data_table_lowering;
pub mod hyperlink_lowering;
pub mod merge_lowering;
pub mod name_lowering;
pub mod pivot_lowering;
pub mod sheet_lowering;
pub mod sparkline_lowering;
pub mod table_lowering;
pub mod validation_lowering;
pub mod view_lowering;

#[cfg(test)]
mod tests;

/// Convert a `ParseOutput` into a `WorkbookSnapshot` for compute-core initialization.
///
/// When `id_map` is `Some`, the snapshot uses the **same IDs** that
/// `hydrate_from_parse_output` allocated into Yrs storage, ensuring a
/// single identity space across Yrs and ComputeCore. When `None`,
/// fast monotonic IDs are generated via `STORAGE_ID_ALLOC`.
pub fn parse_output_to_workbook_snapshot(
    output: &ParseOutput,
    id_map: Option<&HydrationIdMap>,
    allocator: &mut DefaultIdAllocator,
) -> WorkbookSnapshot {
    let mut sheets = sheet_lowering::convert_sheets(&output.sheets, id_map);
    let resolver = SheetResolver::new(&sheets);
    let mut named_ranges = name_lowering::convert_named_ranges(&output.named_ranges, &resolver);
    let tables = table_lowering::convert_tables_from_sheets(&output.sheets, &resolver);
    let pivot_tables = pivot_lowering::convert_pivot_tables(output, &resolver);
    let data_table_regions = data_table_lowering::convert_data_table_regions(output, &resolver);
    let (iterative_calc, max_iterations, max_change) =
        sheet_lowering::convert_iterative_calc(&output.calculation);

    // Run the import classifier when id_map is available (production import paths).
    // The classifier detects homogeneous column runs and converts them to RangeData.
    if let Some(id_map) = id_map {
        // Build a lightweight snapshot for anchor collection (cross-sheet lookups).
        // Sheets are excluded — we iterate them mutably below.
        let snapshot_so_far = WorkbookSnapshot {
            named_ranges: named_ranges.clone(),
            tables: tables.clone(),
            pivot_tables: pivot_tables.clone(),
            data_table_regions: data_table_regions.clone(),
            iterative_calc,
            max_iterations,
            max_change,
            ..WorkbookSnapshot::default()
        };

        // Lazy: only build the reverse cell-id-to-position lookup when named
        // ranges exist (they are the only cross-sheet feature that needs it).
        let cell_id_to_pos: Option<rustc_hash::FxHashMap<String, (u32, u32)>> =
            if named_ranges.is_empty() {
                None
            } else {
                let mut map = rustc_hash::FxHashMap::default();
                for sheet in &sheets {
                    for cell in &sheet.cells {
                        map.insert(cell.cell_id.clone(), (cell.row, cell.col));
                    }
                }
                Some(map)
            };

        debug_assert_eq!(id_map.row_ids.len(), sheets.len());
        debug_assert_eq!(id_map.col_ids.len(), sheets.len());
        debug_assert_eq!(output.sheets.len(), sheets.len());

        for (sheet_idx, sheet) in sheets.iter_mut().enumerate() {
            if sheet_idx < id_map.row_ids.len()
                && sheet_idx < id_map.col_ids.len()
                && sheet_idx < output.sheets.len()
            {
                classifier::classify_sheet_ranges(
                    sheet,
                    &output.sheets[sheet_idx],
                    &snapshot_so_far,
                    cell_id_to_pos.as_ref(),
                    &id_map.row_ids[sheet_idx],
                    &id_map.col_ids[sheet_idx],
                    allocator,
                );
            }
        }

        // Best-effort linkage: link named ranges to Data Ranges that cover
        // the same column/row region.
        name_lowering::link_named_ranges_to_data_ranges(
            &mut named_ranges,
            &sheets,
            &id_map.row_ids,
            &id_map.col_ids,
        );
    }

    WorkbookSnapshot {
        sheets,
        named_ranges,
        tables,
        pivot_tables,
        data_table_regions,
        iterative_calc,
        max_iterations,
        max_change,
        calculation_settings: Some(output.calculation.clone().into()),
    }
}

/// Lower one newly parsed sheet into an existing cumulative snapshot.
///
/// The deferred preview and hydration paths replace exactly one entry in
/// `ParseOutput.sheets` on each request. This entry point keeps every other
/// lowered `SheetSnapshot` (including its `RangeData` identities) intact, then
/// runs the classifier only for `sheet_index` before refreshing the small
/// workbook-level metadata pieces affected by that replacement.
///
/// `previous` is the authoritative cumulative lowered form. The caller must
/// pass the same sheet inventory and identity map that were used to allocate
/// the current cumulative parse. The full lowering entry point above remains
/// the one-shot path for initial import and complete deferred hydration.
pub fn parse_output_to_workbook_snapshot_incremental(
    output: &ParseOutput,
    sheet_index: usize,
    id_map: &HydrationIdMap,
    previous: &WorkbookSnapshot,
    allocator: &mut DefaultIdAllocator,
) -> WorkbookSnapshot {
    debug_assert_eq!(output.sheets.len(), previous.sheets.len());
    debug_assert!(sheet_index < output.sheets.len());
    debug_assert_eq!(id_map.sheet_ids.len(), output.sheets.len());
    debug_assert_eq!(id_map.row_ids.len(), output.sheets.len());
    debug_assert_eq!(id_map.col_ids.len(), output.sheets.len());

    // Resolver metadata needs only stable IDs, names, and positions. Building
    // headers avoids lowering cells from every already-visited sheet merely to
    // resolve a table or pivot's sheet target.
    let resolver_sheets = sheet_lowering::convert_sheet_headers(&output.sheets, id_map);
    let resolver = SheetResolver::new(&resolver_sheets);
    let target_sheet_id = resolver
        .by_index(sheet_index)
        .expect("incremental snapshot target sheet must have an allocated ID");

    // Named ranges and data-table definitions come from workbook-level parse
    // metadata, which is unchanged when the caller replaces only one sheet.
    // Named-range linkage is refreshed below because the target sheet's
    // classifier ranges may have changed.
    let mut named_ranges = previous.named_ranges.clone();
    for named_range in &mut named_ranges {
        named_range.linked_range_id = None;
    }
    let tables = incremental_tables(output, sheet_index, &resolver, previous);
    let pivot_tables = incremental_pivot_tables(
        output,
        &resolver,
        target_sheet_id,
        &output.sheets[sheet_index].name,
        previous,
    );
    let data_table_regions = previous.data_table_regions.clone();
    let (iterative_calc, max_iterations, max_change) =
        sheet_lowering::convert_iterative_calc(&output.calculation);

    // The classifier's cross-sheet anchor inputs are workbook metadata, not
    // lowered sheet cells. It does need a reverse identity lookup for named
    // ranges, so carry forward the previous sparse cells and replace only the
    // target's entries in that lookup.
    let mut target_sheet =
        sheet_lowering::convert_sheet(sheet_index, &output.sheets[sheet_index], Some(id_map));
    let cell_id_to_pos = if named_ranges.is_empty() {
        None
    } else {
        let mut map = FxHashMap::default();
        for (existing_index, sheet) in previous.sheets.iter().enumerate() {
            if existing_index == sheet_index {
                continue;
            }
            for cell in &sheet.cells {
                map.insert(cell.cell_id.clone(), (cell.row, cell.col));
            }
        }
        for cell in &target_sheet.cells {
            map.insert(cell.cell_id.clone(), (cell.row, cell.col));
        }
        Some(map)
    };

    let snapshot_so_far = WorkbookSnapshot {
        named_ranges: named_ranges.clone(),
        tables: tables.clone(),
        pivot_tables: pivot_tables.clone(),
        data_table_regions: data_table_regions.clone(),
        iterative_calc,
        max_iterations,
        max_change,
        ..WorkbookSnapshot::default()
    };
    classifier::classify_sheet_ranges(
        &mut target_sheet,
        &output.sheets[sheet_index],
        &snapshot_so_far,
        cell_id_to_pos.as_ref(),
        &id_map.row_ids[sheet_index],
        &id_map.col_ids[sheet_index],
        allocator,
    );

    let mut sheets = previous.sheets.clone();
    sheets[sheet_index] = target_sheet;
    name_lowering::link_named_ranges_to_data_ranges(
        &mut named_ranges,
        &sheets,
        &id_map.row_ids,
        &id_map.col_ids,
    );

    WorkbookSnapshot {
        sheets,
        named_ranges,
        tables,
        pivot_tables,
        data_table_regions,
        iterative_calc,
        max_iterations,
        max_change,
        calculation_settings: Some(output.calculation.clone().into()),
    }
}

fn incremental_tables(
    output: &ParseOutput,
    sheet_index: usize,
    resolver: &SheetResolver<'_>,
    previous: &WorkbookSnapshot,
) -> Vec<formula_types::TableDef> {
    let mut tables = Vec::new();
    for (index, sheet_data) in output.sheets.iter().enumerate() {
        if index == sheet_index {
            tables.extend(table_lowering::convert_tables_for_sheet(
                index, sheet_data, resolver,
            ));
            continue;
        }

        let Some(sheet_uuid) = resolver
            .by_index(index)
            .and_then(|uuid| cell_types::SheetId::from_uuid_str(uuid).ok())
        else {
            continue;
        };
        tables.extend(
            previous
                .tables
                .iter()
                .filter(|table| table.sheet == sheet_uuid)
                .cloned(),
        );
    }
    tables
}

fn incremental_pivot_tables(
    output: &ParseOutput,
    resolver: &SheetResolver<'_>,
    target_sheet_id: &str,
    target_sheet_name: &str,
    previous: &WorkbookSnapshot,
) -> Vec<snapshot_types::PivotTableDef> {
    let mut pivots: Vec<_> = previous
        .pivot_tables
        .iter()
        .filter(|pivot| pivot.sheet != target_sheet_id)
        .cloned()
        .collect();
    pivots.extend(pivot_lowering::convert_pivot_tables_for_sheet(
        output,
        resolver,
        target_sheet_name,
    ));
    pivots
}

// =============================================================================
// Sheet ID resolution
// =============================================================================

/// Resolves sheet names and indices to UUID strings from the converted
/// `SheetSnapshot` array. Centralises the name→UUID and index→UUID lookups
/// so every converter uses the same resolution logic.
pub(crate) struct SheetResolver<'a> {
    sheets: &'a [SheetSnapshot],
}

impl<'a> SheetResolver<'a> {
    pub(crate) fn new(sheets: &'a [SheetSnapshot]) -> Self {
        Self { sheets }
    }

    /// Resolve a sheet by its 0-based index in the original `ParseOutput.sheets`.
    pub(crate) fn by_index(&self, idx: usize) -> Option<&'a str> {
        self.sheets.get(idx).map(|s| s.id.as_str())
    }

    /// Resolve a sheet by its display name (case-sensitive, matching XLSX names).
    pub(crate) fn by_name(&self, name: &str) -> Option<&'a str> {
        self.sheets
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.id.as_str())
    }
}
