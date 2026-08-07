use super::*;

use std::collections::HashSet;

use cell_types::SheetId;
use value_types::ComputeError;

pub(in crate::storage::engine) type XlsxHydrateResult = (
    YrsStorage,
    WorkbookSnapshot,
    Vec<(SheetId, CellId, u32, u32)>,
    domain_types::ImportReport,
);

/// Data stored for deferred Yrs CRDT hydration.
/// After the fast-path import, this holds everything needed to complete
/// the Yrs write and rebuild indexes with full fidelity.
pub struct DeferredHydrationData {
    pub(in crate::storage::engine) parse_output: domain_types::ParseOutput,
    pub(in crate::storage::engine) allocations:
        Vec<crate::storage::infra::hydration::SheetIdAllocation>,
    pub(in crate::storage::engine) workbook_snap: WorkbookSnapshot,
    /// Raw XLSX bytes for full re-parse during deferred hydration.
    /// The fast-path parse uses values_only + skip options; the full parse
    /// during hydration needs the complete data.
    pub(in crate::storage::engine) raw_xlsx_bytes: Option<Vec<u8>>,

    /// The workbook inventory captured at import time. This is the authority
    /// for deciding whether a deferred XLSX import is complete; the presence
    /// of a sheet in Yrs metadata alone is not enough.
    pub(in crate::storage::engine) expected_imported_sheets: HashSet<SheetId>,
    /// Sheets whose values, axes, mirror entries, and sheet-local indexes are
    /// available to the preview/read path.
    pub(in crate::storage::engine) mirror_materialized_sheets: HashSet<SheetId>,
    /// Sheets whose canonical Yrs subtrees have been durably hydrated.
    pub(in crate::storage::engine) yrs_hydrated_sheets: HashSet<SheetId>,
}

/// Actionable details for an incomplete deferred-hydration guard.
///
/// `ComputeError` currently carries a human-readable message at this layer, so
/// keep the fields structured until the final conversion. This prevents the
/// guard from losing the missing-sheet inventory while still preserving the
/// existing bridge error contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::storage::engine) struct DeferredHydrationIncomplete {
    pub operation: String,
    pub missing_count: usize,
    pub missing_sheets: Vec<String>,
    pub inconsistent: bool,
}

impl DeferredHydrationIncomplete {
    pub fn into_compute_error(self) -> ComputeError {
        ComputeError::InvalidInput {
            message: self.to_string(),
        }
    }
}

impl std::fmt::Display for DeferredHydrationIncomplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inconsistent {
            write!(
                f,
                "{operation} found inconsistent deferred XLSX hydration state; action=complete_deferred_hydration; missing_count={missing_count}; missing_sheets={missing_sheets:?}",
                operation = self.operation,
                missing_count = self.missing_count,
                missing_sheets = self.missing_sheets,
            )
        } else {
            write!(
                f,
                "{operation} requires deferred XLSX hydration to complete; action=complete_deferred_hydration; missing_count={missing_count}; missing_sheets={missing_sheets:?}",
                operation = self.operation,
                missing_count = self.missing_count,
                missing_sheets = self.missing_sheets,
            )
        }
    }
}

impl DeferredHydrationData {
    /// True only when the explicit state sets describe a valid monotonic state.
    pub(in crate::storage::engine) fn hydration_state_is_consistent(&self) -> bool {
        let allocated_sheets: HashSet<SheetId> = self
            .allocations
            .iter()
            .map(|allocation| allocation.sheet_id)
            .collect();
        self.expected_imported_sheets == allocated_sheets
            && self
                .yrs_hydrated_sheets
                .is_subset(&self.mirror_materialized_sheets)
            && self
                .mirror_materialized_sheets
                .is_subset(&self.expected_imported_sheets)
    }

    /// The exact completion predicate for an incremental import.
    pub(in crate::storage::engine) fn all_expected_sheets_yrs_hydrated(&self) -> bool {
        self.hydration_state_is_consistent()
            && self.yrs_hydrated_sheets == self.expected_imported_sheets
    }

    pub(in crate::storage::engine) fn sheet_is_mirror_materialized(
        &self,
        sheet_id: &SheetId,
    ) -> bool {
        self.mirror_materialized_sheets.contains(sheet_id)
    }

    pub(in crate::storage::engine) fn sheet_is_yrs_hydrated(&self, sheet_id: &SheetId) -> bool {
        self.yrs_hydrated_sheets.contains(sheet_id)
    }

    pub(in crate::storage::engine) fn missing_yrs_sheets(&self) -> Vec<SheetId> {
        self.allocations
            .iter()
            .map(|allocation| allocation.sheet_id)
            .filter(|sheet_id| !self.yrs_hydrated_sheets.contains(sheet_id))
            .collect()
    }

    pub(in crate::storage::engine) fn incomplete_error(
        &self,
        operation: &str,
    ) -> DeferredHydrationIncomplete {
        let missing_sheets = self
            .missing_yrs_sheets()
            .into_iter()
            .take(8)
            .map(|sheet_id| {
                self.allocations
                    .iter()
                    .position(|allocation| allocation.sheet_id == sheet_id)
                    .and_then(|index| self.parse_output.sheets.get(index))
                    .map(|sheet| format!("{} ({sheet_id})", sheet.name))
                    .unwrap_or_else(|| sheet_id.to_string())
            })
            .collect();

        DeferredHydrationIncomplete {
            operation: operation.to_string(),
            missing_count: self.expected_imported_sheets.len().saturating_sub(
                self.yrs_hydrated_sheets
                    .intersection(&self.expected_imported_sheets)
                    .count(),
            ),
            missing_sheets,
            inconsistent: !self.hydration_state_is_consistent(),
        }
    }
}

/// Fully staged deferred XLSX completion. This owns every component needed to
/// replace the live engine after any fallible import-open recalculation has
/// succeeded.
pub(in crate::storage::engine) struct DeferredHydrationCompletion {
    pub(in crate::storage::engine) stores: EngineStores,
    pub(in crate::storage::engine) mirror: CellMirror,
    pub(in crate::storage::engine) settings: EngineSettings,
    pub(in crate::storage::engine) phantom_cells: Vec<(SheetId, CellId, u32, u32)>,
    pub(in crate::storage::engine) calculation: domain_types::CalculationProperties,
    pub(in crate::storage::engine) import_report: domain_types::ImportReport,
}
