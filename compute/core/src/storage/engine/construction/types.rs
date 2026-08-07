use super::*;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

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
    /// Workbook-wide, cell-free dependency inventory used to admit edits
    /// without constructing a graph from parse-only sheets.
    pub(in crate::storage::engine) sheet_dependency_manifest: SheetDependencyManifest,
    /// Successful row-4 cell edits that must be replayed if the legacy full
    /// hydration path later rebuilds the document from the retained XLSX.
    /// Cell identities are captured after the live mutation succeeds.
    pub(in crate::storage::engine) committed_cell_edit_batches: Vec<DeferredCommittedCellEditBatch>,
}

#[derive(Clone)]
pub(in crate::storage::engine) struct DeferredCommittedCellEdit {
    pub sheet_id: SheetId,
    pub cell_id: CellId,
    pub row: u32,
    pub col: u32,
    pub input: crate::storage::engine::mutation::CellInput,
}

#[derive(Clone, Copy)]
pub(in crate::storage::engine) enum DeferredCommittedCellEditKind {
    SetCell,
    SetCells,
    SetCellsByPosition,
}

#[derive(Clone)]
pub(in crate::storage::engine) struct DeferredCommittedCellEditBatch {
    pub edits: Vec<DeferredCommittedCellEdit>,
    pub skip_cycle_check: bool,
    pub kind: DeferredCommittedCellEditKind,
}

/// Parser dependency facts translated from workbook-order indices to the
/// stable `SheetId`s allocated for this engine import.
pub(in crate::storage::engine) struct SheetDependencyManifest {
    pub parsed: xlsx_api::SheetDependencyManifest,
    pub workbook_index_by_sheet: HashMap<SheetId, u32>,
    pub sheet_by_workbook_index: BTreeMap<u32, SheetId>,
    pub precedents_by_dependent: HashMap<SheetId, HashSet<SheetId>>,
    pub blockers_by_dependent: HashMap<SheetId, BTreeSet<xlsx_api::DependencyBlocker>>,
    pub sheet_order: Vec<SheetId>,
}

impl SheetDependencyManifest {
    pub fn from_parser_manifest(
        parsed: xlsx_api::SheetDependencyManifest,
        parse_output: &domain_types::ParseOutput,
        allocations: &[crate::storage::infra::hydration::SheetIdAllocation],
    ) -> Result<Self, ComputeError> {
        let mut workbook_index_by_sheet = HashMap::with_capacity(allocations.len());
        let mut sheet_by_workbook_index = BTreeMap::new();
        for (editable_index, allocation) in allocations.iter().enumerate() {
            let workbook_index = parse_output
                .workbook_sheet_inventory
                .iter()
                .find(|entry| entry.editable_sheet_index == Some(editable_index))
                .map(|entry| entry.workbook_order)
                .ok_or_else(|| ComputeError::Deserialize {
                    message: format!(
                        "dependency manifest could not map editable sheet index {editable_index}"
                    ),
                })?;
            workbook_index_by_sheet.insert(allocation.sheet_id, workbook_index);
            sheet_by_workbook_index.insert(workbook_index, allocation.sheet_id);
        }

        let mut precedents_by_dependent = HashMap::<SheetId, HashSet<SheetId>>::new();
        let mut blockers_by_dependent =
            HashMap::<SheetId, BTreeSet<xlsx_api::DependencyBlocker>>::new();
        for (dependent_index, precedent_indices) in &parsed.precedents_by_dependent {
            let Some(dependent) = sheet_by_workbook_index.get(dependent_index).copied() else {
                continue;
            };
            for precedent_index in precedent_indices {
                if let Some(precedent) = sheet_by_workbook_index.get(precedent_index).copied() {
                    if precedent != dependent {
                        precedents_by_dependent
                            .entry(dependent)
                            .or_default()
                            .insert(precedent);
                    }
                } else {
                    blockers_by_dependent
                        .entry(dependent)
                        .or_default()
                        .insert(xlsx_api::DependencyBlocker::UnresolvedSheetReference);
                }
            }
        }
        for (dependent_index, blockers) in &parsed.blockers_by_dependent {
            if let Some(dependent) = sheet_by_workbook_index.get(dependent_index).copied() {
                blockers_by_dependent
                    .entry(dependent)
                    .or_default()
                    .extend(blockers.iter().cloned());
            }
        }

        Ok(Self {
            parsed,
            workbook_index_by_sheet,
            sheet_by_workbook_index,
            precedents_by_dependent,
            blockers_by_dependent,
            sheet_order: allocations
                .iter()
                .map(|allocation| allocation.sheet_id)
                .collect(),
        })
    }
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
    pub(in crate::storage::engine) committed_cell_edit_batches: Vec<DeferredCommittedCellEditBatch>,
}
