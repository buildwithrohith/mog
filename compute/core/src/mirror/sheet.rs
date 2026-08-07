//! Sheet-level CRUD operations for the cell mirror.

use cell_types::SheetId;

use super::cell_mirror::CellMirror;
use super::sheet_key::normalize_sheet_key;
use super::types::SheetMirror;

impl CellMirror {
    /// Replace one sheet's mirror subtree without rebuilding workbook-level
    /// caches or any other sheet. The candidate is built in a temporary mirror
    /// first so malformed identities cannot disturb the live sheet.
    pub fn replace_sheet(
        &mut self,
        snapshot: crate::snapshot::SheetSnapshot,
    ) -> Result<(), value_types::ComputeError> {
        let sheet_id = SheetId::from_uuid_str(&snapshot.id)?;
        let mut candidate = CellMirror::new();
        candidate.add_sheet(snapshot)?;

        // `add_sheet` also builds the target sheet's projection and CSE
        // metadata. Preserve those target-local derived indexes while keeping
        // every other sheet's registry entry in the live mirror.
        let replacement_projections: Vec<_> = candidate
            .projection_registry
            .iter_projections()
            .filter(|(_, projection)| projection.sheet == sheet_id)
            .map(|(_, projection)| projection.clone())
            .collect();
        let replacement_cse_single_cell = candidate.cse_single_cell.clone();
        let replacement_cse_anchors = candidate.cse_anchors.clone();

        let replacement = candidate
            .sheets
            .remove(&sheet_id)
            .expect("candidate mirror just inserted the replacement sheet");
        let previous = self.take_sheet(&sheet_id);
        let mut replacement = replacement;
        if let Some(previous) = previous {
            // SheetSnapshot carries cells and ranges, but target-local domain
            // caches are hydrated from Yrs separately. Keep those caches when
            // replacing the value/index subtree so comments, merges,
            // dimensions, and format ranges do not disappear from the mirror.
            preserve_sheet_domain_caches(&mut replacement, previous);
        }
        self.row_to_sheet.retain(|_, owner| *owner != sheet_id);
        self.col_to_sheet.retain(|_, owner| *owner != sheet_id);
        self.sheet_names
            .insert(normalize_sheet_key(&replacement.name), sheet_id);
        for cell_id in replacement.cells.keys() {
            self.cell_to_sheet.insert(*cell_id, sheet_id);
        }
        self.sheets.insert(sheet_id, replacement);
        self.col_versions.retain(|(sid, _), _| *sid != sheet_id);
        self.dense_cache.invalidate_sheet(&sheet_id);
        self.cse_single_cell.extend(replacement_cse_single_cell);
        self.cse_anchors.extend(replacement_cse_anchors);
        for projection in replacement_projections {
            self.projection_registry.register(
                projection.source,
                projection.sheet,
                projection.origin_row,
                projection.origin_col,
                projection.rows,
                projection.cols,
            );
        }
        Ok(())
    }

    /// Remove a sheet by SheetId.
    pub fn remove_sheet(&mut self, sheet: &SheetId) {
        let _ = self.take_sheet(sheet);
    }

    /// Rename a sheet.
    pub fn rename_sheet(&mut self, sheet: &SheetId, name: &str) {
        if let Some(s) = self.sheets.get_mut(sheet) {
            // Remove old name mapping
            self.sheet_names.remove(&normalize_sheet_key(&s.name));
            // Update sheet name
            s.name = name.to_string();
            // Insert new name mapping
            self.sheet_names.insert(normalize_sheet_key(name), *sheet);
        }
    }
}

impl CellMirror {
    fn take_sheet(&mut self, sheet: &SheetId) -> Option<SheetMirror> {
        let sheet_mirror = self.sheets.remove(sheet)?;
        self.sheet_names
            .remove(&normalize_sheet_key(&sheet_mirror.name));
        self.cell_to_sheet.retain(|_, owner| owner != sheet);
        self.row_to_sheet.retain(|_, owner| owner != sheet);
        self.col_to_sheet.retain(|_, owner| owner != sheet);
        self.col_versions.retain(|(sid, _), _| sid != sheet);
        for cell_id in sheet_mirror.cells.keys() {
            self.cse_single_cell.remove(cell_id);
            self.cse_anchors.remove(cell_id);
        }
        let projection_sources: Vec<_> = self
            .projection_registry
            .iter_projections()
            .filter(|(_, projection)| projection.sheet == *sheet)
            .map(|(source, _)| *source)
            .collect();
        for source in projection_sources {
            self.projection_registry.remove(&source);
        }
        self.dense_cache.invalidate_sheet(sheet);
        Some(sheet_mirror)
    }
}

fn preserve_sheet_domain_caches(replacement: &mut SheetMirror, previous: SheetMirror) {
    replacement.merge_regions = previous.merge_regions;
    replacement.row_heights = previous.row_heights;
    replacement.col_widths = previous.col_widths;
    replacement.hidden_rows = previous.hidden_rows;
    replacement.hidden_cols = previous.hidden_cols;
    replacement.comment_cells = previous.comment_cells;
    replacement.sparkline_cells = previous.sparkline_cells;
    replacement.enable_calculation = previous.enable_calculation;
    replacement.format_ranges = previous.format_ranges;
    replacement.format_range_spatial_index = previous.format_range_spatial_index;
    replacement.range_format_cache = previous.range_format_cache;
    replacement.range_xlsx_style_id_cache = previous.range_xlsx_style_id_cache;
    replacement.col_format_ranges = previous.col_format_ranges;
    replacement.col_format_range_spatial_index = previous.col_format_range_spatial_index;
    replacement.col_format_range_cache = previous.col_format_range_cache;
    replacement.col_range_xlsx_style_id_cache = previous.col_range_xlsx_style_id_cache;
}
