use super::super::YrsComputeEngine;
use super::super::services;
use super::yrs_position_persistence::persist_remapped_cell_positions;
use crate::mirror::CellMirror;
use crate::snapshot::{MutationResult, RecalcResult};
use cell_types::{CellId, SheetId};
use compute_document::undo::ORIGIN_STRUCTURAL;
use compute_wire::mutation::serialize_multi_viewport_patches;
use formula_types::StructureChange;
use value_types::ComputeError;

impl YrsComputeEngine {
    pub(super) fn apply_insert_cells_with_shift(
        &mut self,
        sheet_id: &SheetId,
        row: u32,
        col: u32,
        row_count: u32,
        col_count: u32,
        shift_right: bool,
    ) -> Result<(Vec<u8>, MutationResult), ComputeError> {
        if row_count == 0 || col_count == 0 {
            return Ok((
                serialize_multi_viewport_patches(&[]),
                MutationResult::empty(),
            ));
        }

        self.ensure_partial_cell_shift_supported(sheet_id, "insert_cells_with_shift")?;
        let updates = self.collect_insert_cell_shift_updates(
            sheet_id,
            row,
            col,
            row_count,
            col_count,
            shift_right,
        )?;
        self.apply_partial_insert_shift(sheet_id, updates)
    }

    pub(super) fn apply_delete_cells_with_shift(
        &mut self,
        sheet_id: &SheetId,
        row: u32,
        col: u32,
        row_count: u32,
        col_count: u32,
        shift_left: bool,
    ) -> Result<(Vec<u8>, MutationResult), ComputeError> {
        if row_count == 0 || col_count == 0 {
            return Ok((
                serialize_multi_viewport_patches(&[]),
                MutationResult::empty(),
            ));
        }

        self.ensure_partial_cell_shift_supported(sheet_id, "delete_cells_with_shift")?;
        let (deleted_cell_ids, updates) = self
            .collect_delete_cell_shift_plan(sheet_id, row, col, row_count, col_count, shift_left)?;
        self.apply_partial_delete_shift(sheet_id, deleted_cell_ids, updates)
    }

    fn ensure_partial_cell_shift_supported(
        &self,
        sheet_id: &SheetId,
        operation: &str,
    ) -> Result<(), ComputeError> {
        if self
            .mirror
            .get_sheet(sheet_id)
            .is_some_and(|s| !s.range_views_is_empty())
        {
            return Err(ComputeError::RangeGuardViolation {
                sheet_id: sheet_id.to_uuid_string(),
                operation: operation.to_string(),
            });
        }
        Ok(())
    }

    fn collect_insert_cell_shift_updates(
        &self,
        sheet_id: &SheetId,
        row: u32,
        col: u32,
        row_count: u32,
        col_count: u32,
        shift_right: bool,
    ) -> Result<Vec<(CellId, u32, u32)>, ComputeError> {
        let grid =
            self.stores
                .grid_indexes
                .get(sheet_id)
                .ok_or_else(|| ComputeError::SheetNotFound {
                    sheet_id: sheet_id.to_uuid_string(),
                })?;

        let mut updates = Vec::new();
        if shift_right {
            let row_end = row.saturating_add(row_count).min(grid.row_count());
            let col_limit = grid.col_count();
            if col >= col_limit {
                return Ok(updates);
            }
            for r in row..row_end {
                for c in (col..col_limit).rev() {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        updates.push((cell_id, r, c.saturating_add(col_count)));
                    }
                }
            }
        } else {
            let col_end = col.saturating_add(col_count).min(grid.col_count());
            let row_limit = grid.row_count();
            if row >= row_limit {
                return Ok(updates);
            }
            for c in col..col_end {
                for r in (row..row_limit).rev() {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        updates.push((cell_id, r.saturating_add(row_count), c));
                    }
                }
            }
        }
        Ok(updates)
    }

    fn collect_delete_cell_shift_plan(
        &self,
        sheet_id: &SheetId,
        row: u32,
        col: u32,
        row_count: u32,
        col_count: u32,
        shift_left: bool,
    ) -> Result<(Vec<CellId>, Vec<(CellId, u32, u32)>), ComputeError> {
        let grid =
            self.stores
                .grid_indexes
                .get(sheet_id)
                .ok_or_else(|| ComputeError::SheetNotFound {
                    sheet_id: sheet_id.to_uuid_string(),
                })?;

        let mut deleted_cell_ids = Vec::new();
        let mut updates = Vec::new();

        if shift_left {
            let row_end = row.saturating_add(row_count).min(grid.row_count());
            let delete_end = col.saturating_add(col_count).min(grid.col_count());
            let col_limit = grid.col_count();
            if col >= col_limit {
                return Ok((deleted_cell_ids, updates));
            }
            for r in row..row_end {
                for c in col..delete_end {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        deleted_cell_ids.push(cell_id);
                    }
                }
                for c in delete_end..col_limit {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        updates.push((cell_id, r, c.saturating_sub(col_count)));
                    }
                }
            }
        } else {
            let col_end = col.saturating_add(col_count).min(grid.col_count());
            let delete_end = row.saturating_add(row_count).min(grid.row_count());
            let row_limit = grid.row_count();
            if row >= row_limit {
                return Ok((deleted_cell_ids, updates));
            }
            for c in col..col_end {
                for r in row..delete_end {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        deleted_cell_ids.push(cell_id);
                    }
                }
                for r in delete_end..row_limit {
                    if let Some(cell_id) = grid.cell_id_at(r, c) {
                        updates.push((cell_id, r.saturating_sub(row_count), c));
                    }
                }
            }
        }

        Ok((deleted_cell_ids, updates))
    }

    fn apply_partial_insert_shift(
        &mut self,
        sheet_id: &SheetId,
        updates: Vec<(CellId, u32, u32)>,
    ) -> Result<(Vec<u8>, MutationResult), ComputeError> {
        let recalc = self.apply_partial_cell_remap(sheet_id, updates)?;
        self.security.bump_structure_version();
        self.finish_structure_change(sheet_id, recalc, None)
    }

    fn apply_partial_delete_shift(
        &mut self,
        sheet_id: &SheetId,
        deleted_cell_ids: Vec<CellId>,
        updates: Vec<(CellId, u32, u32)>,
    ) -> Result<(Vec<u8>, MutationResult), ComputeError> {
        let recalc =
            self.apply_partial_cell_delete_and_remap(sheet_id, deleted_cell_ids, updates)?;
        self.security.bump_structure_version();
        self.finish_structure_change(sheet_id, recalc, None)
    }

    fn apply_partial_cell_remap(
        &mut self,
        sheet_id: &SheetId,
        updates: Vec<(CellId, u32, u32)>,
    ) -> Result<RecalcResult, ComputeError> {
        if updates.is_empty() {
            return Ok(RecalcResult::empty());
        }

        let old_position_keys = self.old_position_keys_for_updates(sheet_id, &updates)?;
        let change = StructureChange::RemapPositions { updates };
        self.mutation.undo_manager.begin_undo_group();
        let _guard = self.mutation.suppress_guard();
        let apply_result = services::structural::apply_structure_change(
            &mut self.stores,
            &mut self.mirror,
            sheet_id,
            &change,
        );
        let persist_result = apply_result.and_then(|recalc| {
            persist_remapped_cell_positions(
                &self.stores,
                sheet_id,
                match &change {
                    StructureChange::RemapPositions { updates } => updates,
                    _ => unreachable!(),
                },
                &old_position_keys,
            )?;
            Ok(recalc)
        });
        drop(_guard);
        self.mutation.undo_manager.end_undo_group();
        persist_result
    }

    fn apply_partial_cell_delete_and_remap(
        &mut self,
        sheet_id: &SheetId,
        deleted_cell_ids: Vec<CellId>,
        updates: Vec<(CellId, u32, u32)>,
    ) -> Result<RecalcResult, ComputeError> {
        if deleted_cell_ids.is_empty() && updates.is_empty() {
            return Ok(RecalcResult::empty());
        }

        let old_position_keys = self.old_position_keys_for_updates(sheet_id, &updates)?;
        let deleted_position_keys =
            self.old_position_keys_for_cell_ids(sheet_id, &deleted_cell_ids)?;
        self.mutation.undo_manager.begin_undo_group();
        let _guard = self.mutation.suppress_guard();

        let clear_result = if deleted_cell_ids.is_empty() {
            Ok(())
        } else {
            Self::clear_cells_for_partial_structural_delete(
                &mut self.stores,
                &mut self.mirror,
                sheet_id,
                &deleted_position_keys,
            )
        };

        let recalc_result = match clear_result {
            Err(err) => Err(err),
            Ok(()) if updates.is_empty() => {
                self.stores.compute.structure_change(&mut self.mirror, None)
            }
            Ok(()) => {
                let change = StructureChange::RemapPositions { updates };
                let recalc = services::structural::apply_structure_change(
                    &mut self.stores,
                    &mut self.mirror,
                    sheet_id,
                    &change,
                )?;
                persist_remapped_cell_positions(
                    &self.stores,
                    sheet_id,
                    match &change {
                        StructureChange::RemapPositions { updates } => updates,
                        _ => unreachable!(),
                    },
                    &old_position_keys,
                )?;
                Ok(recalc)
            }
        };

        drop(_guard);
        self.mutation.undo_manager.end_undo_group();
        recalc_result
    }

    fn old_position_keys_for_updates(
        &self,
        sheet_id: &SheetId,
        updates: &[(CellId, u32, u32)],
    ) -> Result<Vec<(CellId, String)>, ComputeError> {
        let grid =
            self.stores
                .grid_indexes
                .get(sheet_id)
                .ok_or_else(|| ComputeError::SheetNotFound {
                    sheet_id: sheet_id.to_uuid_string(),
                })?;
        updates
            .iter()
            .map(|(cell_id, _, _)| {
                let (row, col) =
                    grid.cell_position(cell_id)
                        .ok_or_else(|| ComputeError::InvalidInput {
                            message: format!(
                                "missing current position for remapped cell {}",
                                cell_id.to_uuid_string()
                            ),
                        })?;
                let row_hex = grid
                    .row_id_hex(row)
                    .ok_or_else(|| ComputeError::InvalidInput {
                        message: format!("missing row identity for remapped row {row}"),
                    })?;
                let col_hex = grid
                    .col_id_hex(col)
                    .ok_or_else(|| ComputeError::InvalidInput {
                        message: format!("missing column identity for remapped column {col}"),
                    })?;
                Ok((*cell_id, format!("{row_hex}:{col_hex}")))
            })
            .collect()
    }

    fn old_position_keys_for_cell_ids(
        &self,
        sheet_id: &SheetId,
        cell_ids: &[CellId],
    ) -> Result<Vec<(CellId, String)>, ComputeError> {
        let updates: Vec<(CellId, u32, u32)> = cell_ids
            .iter()
            .copied()
            .map(|cell_id| (cell_id, 0, 0))
            .collect();
        self.old_position_keys_for_updates(sheet_id, &updates)
    }

    fn clear_cells_for_partial_structural_delete(
        stores: &mut super::super::stores::EngineStores,
        mirror: &mut CellMirror,
        sheet_id: &SheetId,
        cell_position_keys: &[(CellId, String)],
    ) -> Result<(), ComputeError> {
        let cell_ids: Vec<CellId> = cell_position_keys
            .iter()
            .map(|(cell_id, _)| *cell_id)
            .collect();
        stores.compute.clear_cells(mirror, &cell_ids)?;
        for (cell_id, pos_key) in cell_position_keys {
            stores.storage.remove_cell_with_origin_at(
                mirror,
                sheet_id,
                cell_id,
                Some(pos_key),
                Some(ORIGIN_STRUCTURAL),
            );
            let grid = stores.grid_indexes.get_mut(sheet_id).ok_or_else(|| {
                ComputeError::SheetNotFound {
                    sheet_id: sheet_id.to_uuid_string(),
                }
            })?;
            grid.remove_cell(cell_id);
        }
        Ok(())
    }
}
