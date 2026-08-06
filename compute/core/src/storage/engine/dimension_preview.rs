//! Session-local row/column dimension overrides for read-only previews.
//!
//! Preview dimensions are deliberately kept outside Yrs. They are only a
//! render/query overlay for the current engine session; durable resize
//! commands remain the sole path that writes canonical points/character
//! widths to the document.

use cell_types::SheetId;
use domain_types::units::Pixels;
use rustc_hash::FxHashMap;

#[derive(Debug, Default)]
pub(crate) struct DimensionPreviewOverlay {
    row_heights: FxHashMap<(SheetId, u32), Pixels>,
    col_widths: FxHashMap<(SheetId, u32), Pixels>,
}

impl DimensionPreviewOverlay {
    pub(crate) fn row_height(&self, sheet_id: &SheetId, row: u32) -> Option<Pixels> {
        self.row_heights.get(&(*sheet_id, row)).copied()
    }

    pub(crate) fn col_width(&self, sheet_id: &SheetId, col: u32) -> Option<Pixels> {
        self.col_widths.get(&(*sheet_id, col)).copied()
    }

    pub(crate) fn set_row_height(&mut self, sheet_id: &SheetId, row: u32, height: Pixels) {
        self.row_heights.insert((*sheet_id, row), height);
    }

    pub(crate) fn set_col_width(&mut self, sheet_id: &SheetId, col: u32, width: Pixels) {
        self.col_widths.insert((*sheet_id, col), width);
    }

    pub(crate) fn clear_row_height(&mut self, sheet_id: &SheetId, row: u32) {
        self.row_heights.remove(&(*sheet_id, row));
    }

    pub(crate) fn clear_col_width(&mut self, sheet_id: &SheetId, col: u32) {
        self.col_widths.remove(&(*sheet_id, col));
    }

    pub(crate) fn clear_sheet(&mut self, sheet_id: &SheetId) {
        self.row_heights.retain(|(sid, _), _| sid != sheet_id);
        self.col_widths.retain(|(sid, _), _| sid != sheet_id);
    }

    pub(crate) fn clear_all(&mut self) {
        self.row_heights.clear();
        self.col_widths.clear();
    }
}
