use super::*;

use std::collections::{HashMap, HashSet, VecDeque};

/// One calculation-bearing cell edit considered before any user transaction.
/// A prospective formula is supplied only when the edit is typing a formula;
/// value edits still use the imported dependency graph rooted at `sheet_id`.
#[derive(Debug, Clone)]
pub(in crate::storage::engine) struct DeferredCalculationEdit {
    pub sheet_id: SheetId,
    pub prospective_formula: Option<String>,
}

impl SheetDependencyManifest {
    fn required_sheets_for_edits(
        &self,
        edits: &[DeferredCalculationEdit],
    ) -> Result<Vec<SheetId>, ComputeError> {
        let mut precedents = self.precedents_by_dependent.clone();
        let mut blockers = self.blockers_by_dependent.clone();
        let mut edited_sheets = HashSet::with_capacity(edits.len());

        for edit in edits {
            let Some(owner_index) = self.workbook_index_by_sheet.get(&edit.sheet_id).copied()
            else {
                return Err(ComputeError::InvalidInput {
                    message: format!(
                        "sheet {} is not part of the deferred dependency manifest",
                        edit.sheet_id
                    ),
                });
            };
            edited_sheets.insert(edit.sheet_id);
            if let Some(formula) = edit.prospective_formula.as_deref() {
                let analysis = self.parsed.analyze_formula(owner_index, formula);
                blockers
                    .entry(edit.sheet_id)
                    .or_default()
                    .extend(analysis.blockers);
                for precedent_index in analysis.precedents {
                    if let Some(precedent) =
                        self.sheet_by_workbook_index.get(&precedent_index).copied()
                    {
                        if precedent != edit.sheet_id {
                            precedents
                                .entry(edit.sheet_id)
                                .or_default()
                                .insert(precedent);
                        }
                    } else {
                        blockers
                            .entry(edit.sheet_id)
                            .or_default()
                            .insert(xlsx_api::DependencyBlocker::UnresolvedSheetReference);
                    }
                }
            }
        }

        // Imported edges are precedent -> dependent. First follow dependents
        // from the edited sheets, then add every precedent needed to evaluate
        // those affected formulas. Do not follow other dependents from a
        // precedent added only for evaluation (Inputs -> DCF <- Revenue must
        // not pull in every report that also happens to use Revenue).
        let mut dependents = HashMap::<SheetId, HashSet<SheetId>>::new();
        for (dependent, precedent_sheets) in &precedents {
            for precedent in precedent_sheets {
                dependents.entry(*precedent).or_default().insert(*dependent);
            }
        }

        let mut affected = edited_sheets.clone();
        let mut queue: VecDeque<SheetId> = edited_sheets.iter().copied().collect();
        while let Some(sheet) = queue.pop_front() {
            for dependent in dependents.get(&sheet).into_iter().flatten() {
                if affected.insert(*dependent) {
                    queue.push_back(*dependent);
                }
            }
        }

        let mut required = affected.clone();
        let mut queue: VecDeque<SheetId> = affected.iter().copied().collect();
        while let Some(sheet) = queue.pop_front() {
            for precedent in precedents.get(&sheet).into_iter().flatten() {
                if required.insert(*precedent) {
                    queue.push_back(*precedent);
                }
            }
        }

        for sheet in &required {
            if let Some(reasons) = blockers.get(sheet)
                && !reasons.is_empty()
            {
                let name = self
                    .workbook_index_by_sheet
                    .get(sheet)
                    .and_then(|index| self.parsed.sheet_names.get(index))
                    .map(String::as_str)
                    .unwrap_or("unknown");
                return Err(ComputeError::InvalidInput {
                    message: format!(
                        "dependency-closure edit admission failed closed for sheet {name} ({sheet}): {reasons:?}"
                    ),
                });
            }
        }

        Ok(self
            .sheet_order
            .iter()
            .copied()
            .filter(|sheet| required.contains(sheet))
            .collect())
    }
}

/// Hydrate and register the exact sheet closure needed by a cell edit.
///
/// The caller must execute the user mutation only after this succeeds, and
/// must scope scheduler admission to the returned sheet IDs. Hydration is
/// monotonic, but a blocker is checked before the first target transaction.
pub(in crate::storage::engine) fn prepare_deferred_calculation_edit(
    engine: &mut YrsComputeEngine,
    edits: &[DeferredCalculationEdit],
) -> Result<Vec<SheetId>, ComputeError> {
    let Some(deferred) = engine.deferred_hydration.as_ref() else {
        return Ok(Vec::new());
    };
    if edits.is_empty() {
        return Ok(Vec::new());
    }

    let required = deferred
        .sheet_dependency_manifest
        .required_sheets_for_edits(edits)?;
    for sheet_id in &required {
        hydrate_deferred_sheet(engine, *sheet_id)?;
    }

    let snapshots = {
        let deferred = engine
            .deferred_hydration
            .as_ref()
            .expect("per-sheet hydration preserves deferred state");
        required
            .iter()
            .filter_map(|sheet_id| {
                deferred
                    .allocations
                    .iter()
                    .position(|allocation| allocation.sheet_id == *sheet_id)
                    .and_then(|index| deferred.workbook_snap.sheets.get(index))
                    .cloned()
            })
            .collect::<Vec<_>>()
    };
    if snapshots.len() != required.len() {
        return Err(ComputeError::Deserialize {
            message: "dependency closure snapshot inventory was incomplete after hydration".into(),
        });
    }
    engine
        .stores
        .compute
        .register_incrementally_hydrated_sheets(&mut engine.mirror, &snapshots)?;

    Ok(required)
}
