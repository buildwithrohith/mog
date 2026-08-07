use super::*;

pub(in crate::storage::engine) fn import_from_xlsx_bytes_deferred(
    engine: &mut YrsComputeEngine,
    xlsx_data: &[u8],
) -> Result<RecalcResult, ComputeError> {
    use crate::import;
    use crate::storage::infra::hydration::{SharedIdAllocator, allocate_sheet_ids};

    // Pass 0: Stream the workbook-wide sheet dependency inventory. This reads
    // formula/name text only and retains no unselected CellData.
    let parsed_dependency_manifest =
        xlsx_api::parse_sheet_dependency_manifest(xlsx_data).map_err(|e| {
            ComputeError::Deserialize {
                message: format!("XLSX dependency manifest error: {e}"),
            }
        })?;

    // Pass 1: Parse XLSX — only the initial active visible sheet's cells
    // (ZIP decompress + XML parse). Remaining sheets get metadata only. Full
    // parse happens in complete_deferred_hydration.
    let parsed = {
        let mut profile = crate::xlsx_profile::PhaseTimer::new("import_deferred", "parse");
        let parsed = xlsx_api::parse_initial_active_visible_sheet(xlsx_data).map_err(|e| {
            ComputeError::Deserialize {
                message: format!("XLSX parse error: {}", e),
            }
        })?;
        profile.counter("sheets", parsed.output.sheets.len() as u64);
        profile.counter(
            "cells",
            parsed
                .output
                .sheets
                .iter()
                .map(|sheet| sheet.cells.len() as u64)
                .sum::<u64>(),
        );
        parsed
    };
    let import_report = parsed.import_report;
    let parse_output = parsed.output;
    let diagnostics = parsed.diagnostics;
    if !diagnostics.errors.is_empty() {
        tracing::warn!(
            error_count = diagnostics.errors.len(),
            "XLSX import produced parse errors"
        );
    }

    // Import replaces the current document contents. The non-deferred path
    // rebuilds a fresh storage instance; the deferred path must do the same
    // before writing the critical first-paint hydration, otherwise callers that
    // import into an already-created blank/default engine can observe stale
    // sheet order and stale workbook maps.
    engine.update_buffer.clear();
    engine.stores.storage = YrsStorage::new();
    engine.import_report = import_report;
    engine.clear_runtime_diagnostics();

    // Pass 2: Allocate IDs for ALL sheets (fast — only ~4ms for 28 sheets).
    // Cell/Row/Col IDs are only allocated for sheets with cells or imported
    // dimensions. Metadata-only sheets still get stable SheetIds.
    let critical_sheet_index = xlsx_api::initial_active_visible_sheet_index(&parse_output)
        .filter(|idx| *idx < parse_output.sheets.len())
        .unwrap_or(0);
    // Keep the allocator used by initial allocation, snapshot lowering, and
    // bootstrap Yrs hydration in one atomic authority. The same Arc becomes
    // the engine's grid/compute allocator below, so any subsequent edit is
    // automatically visible to deferred hydration.
    let mut allocator = SharedIdAllocator::new();
    let allocations: Vec<_> = {
        let mut profile = crate::xlsx_profile::PhaseTimer::new("import_deferred", "id_allocation");
        let allocations: Vec<_> = parse_output
            .sheets
            .iter()
            .map(|sheet| allocate_sheet_ids(sheet, &mut allocator))
            .collect();
        profile.counter("sheets", allocations.len() as u64);
        profile.counter(
            "allocated_cells",
            allocations
                .iter()
                .map(|allocation| allocation.cell_ids.len() as u64)
                .sum::<u64>(),
        );
        profile.counter("critical_sheet_index", critical_sheet_index as u64);
        allocations
    };

    let id_map = {
        use crate::storage::infra::hydration::HydrationIdMap;
        let mut m = HydrationIdMap::default();
        for alloc in &allocations {
            m.sheet_ids.push(alloc.sheet_id);
            m.cell_ids.push(std::sync::Arc::clone(&alloc.cell_ids));
            m.row_ids.push(std::sync::Arc::clone(&alloc.row_ids));
            m.col_ids.push(std::sync::Arc::clone(&alloc.col_ids));
            for identity in &alloc.identity_only_cells {
                m.identity_only_cells.push((
                    alloc.sheet_id,
                    identity.cell_id,
                    identity.row,
                    identity.col,
                ));
            }
        }
        m
    };

    // Pass 3: Build snapshot from the selected parse output. All editable
    // sheets are present for tab strip/order, but only the critical sheet has
    // a full cell/domain payload.
    let workbook_snap = {
        let mut profile = crate::xlsx_profile::PhaseTimer::new(
            "import_deferred",
            "parse_output_to_workbook_snapshot",
        );
        let snap = import::parse_output_to_snapshot::parse_output_to_workbook_snapshot(
            &parse_output,
            Some(&id_map),
            &mut allocator,
        );
        profile.counter("sheets", snap.sheets.len() as u64);
        profile.counter(
            "snapshot_cells",
            snap.sheets
                .iter()
                .map(|sheet| sheet.cells.len() as u64)
                .sum::<u64>(),
        );
        profile.counter(
            "ranges",
            snap.sheets
                .iter()
                .map(|sheet| sheet.ranges.len() as u64)
                .sum::<u64>(),
        );
        snap
    };

    // Pass 5: Hydrate the critical parse output into Yrs.
    //
    // Deferred import still avoids the expensive full-workbook cell hydration:
    // `parse_output` contains all sheet headers but only the critical sheet's
    // cells/domain payloads. Hydrating that data preserves the normal production
    // format read path (`properties::get_effective_format` -> Yrs
    // stylePalette/properties) for first paint, while keeping sheet
    // order/settings coherent for all tabs.
    let mut critical_ranged_positions: Vec<std::collections::HashSet<(u32, u32)>> =
        Vec::with_capacity(parse_output.sheets.len());
    let mut critical_range_data_per_sheet: Vec<Vec<snapshot_types::RangeData>> =
        Vec::with_capacity(parse_output.sheets.len());
    let mut critical_range_style_positions: Vec<std::collections::HashSet<(u32, u32)>> =
        Vec::with_capacity(parse_output.sheets.len());
    let mut critical_range_styles_per_sheet: Vec<
        Vec<crate::storage::infra::hydration::ImportedRangeStyle>,
    > = Vec::with_capacity(parse_output.sheets.len());
    for sheet_idx in 0..parse_output.sheets.len() {
        let plan = match (
            sheet_idx == critical_sheet_index,
            workbook_snap.sheets.get(sheet_idx),
        ) {
            (true, Some(snap_sheet)) => build_deferred_critical_sheet_range_plan(
                &parse_output.sheets[sheet_idx],
                snap_sheet,
                &allocations[sheet_idx],
                &mut allocator,
            ),
            _ => DeferredCriticalSheetRangePlan::default(),
        };
        critical_ranged_positions.push(plan.ranged_positions);
        critical_range_data_per_sheet.push(plan.ranges);
        critical_range_style_positions.push(plan.range_style_positions);
        critical_range_styles_per_sheet.push(plan.range_styles);
    }
    {
        let mut profile = crate::xlsx_profile::PhaseTimer::new(
            "import_deferred",
            "hydrate_from_parse_output_with_ranges",
        );
        let _critical_id_map = engine
            .stores
            .storage
            .hydrate_from_parse_output_with_ranges(
                &parse_output,
                &allocations,
                &critical_ranged_positions,
                &critical_range_style_positions,
                &critical_range_data_per_sheet,
                &critical_range_styles_per_sheet,
                &mut allocator,
            )?;
        engine
            .stores
            .storage
            .hydrate_imported_external_links(&parse_output.external_links)?;
        profile.counter("sheets", parse_output.sheets.len() as u64);
        profile.counter(
            "ranged_positions",
            critical_ranged_positions
                .iter()
                .map(|positions| positions.len() as u64)
                .sum::<u64>(),
        );
        profile.counter(
            "range_style_positions",
            critical_range_style_positions
                .iter()
                .map(|positions| positions.len() as u64)
                .sum::<u64>(),
        );
        profile.counter(
            "range_styles",
            critical_range_styles_per_sheet
                .iter()
                .map(|styles| styles.len() as u64)
                .sum::<u64>(),
        );
    }

    // Build CellMirror + viewport-only compute init.
    // Skips formula extraction entirely (deferred to ensure_graph_built).
    {
        let mut profile =
            crate::xlsx_profile::PhaseTimer::new("import_deferred", "mirror_compute_rebuild");
        engine.stores.compute = ComputeCore::new();
        engine
            .stores
            .compute
            .init_from_snapshot_viewport_only(&mut engine.mirror, &workbook_snap)?;
        profile.counter("sheets", workbook_snap.sheets.len() as u64);
        profile.counter(
            "snapshot_cells",
            workbook_snap
                .sheets
                .iter()
                .map(|sheet| sheet.cells.len() as u64)
                .sum::<u64>(),
        );
    }

    // Pass 8: Build indexes from snapshot/parse_output.
    let shared_alloc = allocator.shared();
    engine.stores.grid_id_alloc = std::sync::Arc::clone(&shared_alloc);
    engine.stores.compute.set_id_alloc(shared_alloc);
    engine.stores.id_alloc =
        std::sync::Arc::new(crate::storage::metadata_id_allocator_for_doc_client(
            engine.stores.storage.doc().client_id(),
        ));

    // Build indexes for only the critical sheet (viewport-visible).
    // Remaining sheets' indexes are built during complete_deferred_hydration.
    let critical_sheet_range = if workbook_snap.sheets.is_empty() {
        0..0
    } else {
        critical_sheet_index..critical_sheet_index.saturating_add(1)
    };
    engine.stores.grid_indexes = build_grid_indexes_from_allocations_range(
        &workbook_snap,
        &allocations,
        critical_sheet_range.clone(),
        engine.stores.grid_id_alloc.clone(),
    )?;
    engine.stores.merge_indexes = build_merge_indexes_from_parse_output_range(
        &parse_output,
        &workbook_snap,
        critical_sheet_range.clone(),
    )?;
    engine.stores.layout_indexes = build_layout_indexes_from_parse_output_range(
        &parse_output,
        &workbook_snap,
        &engine.stores.grid_indexes,
        critical_sheet_range,
        engine.stores.layout_metrics,
    )?;
    engine.stores.dimension_preview.clear_all();

    engine.mirror.install_row_col_indexes(
        engine
            .stores
            .grid_indexes
            .iter()
            .map(|(sid, grid)| (*sid, grid.row_ids_ordered(), grid.col_ids_ordered())),
    );
    hydrate_mirror_format_ranges(&engine.stores.storage, &mut engine.mirror);
    engine.mirror.finalize_range_hydration();
    sync_table_catalog_from_yrs_if_present(&mut engine.stores, &mut engine.mirror);

    crate::storage::engine::services::imported_filters::normalize_imported_auto_filter_visibility(
        &mut engine.stores,
        &mut engine.mirror,
        Some(&mut engine.import_report),
        domain_types::ImportPhase::CriticalSheet,
    );

    // Pass 9: Observer/undo/settings for the critical Yrs document.
    engine.update_buffer.clear();
    engine._update_subscription = crate::storage::engine::update_buffer::install_observer(
        engine.stores.storage.doc(),
        &engine.update_buffer,
        crate::storage::engine::update_buffer::UpdateSource::ImportBootstrap,
    );
    let (observer, undo_manager) = create_observer_and_undo(&engine.stores.storage);
    engine.mutation.observer = observer;
    engine.mutation.undo_manager = undo_manager;
    engine.settings = derive_settings(&engine.stores.storage);
    engine.viewport.clear();

    // Register phantom cells from first sheet
    for (sheet_id, cell_id, row, col) in &id_map.phantom_cells {
        if let Some(grid) = engine.stores.grid_indexes.get_mut(sheet_id) {
            grid.register_cell(*cell_id, *row, *col);
        }
    }

    // Store data for deferred Yrs hydration. `complete_deferred_hydration`
    // re-parses the workbook and re-allocates cells for all sheets before it
    // commits anything to the live engine.
    let expected_imported_sheets = allocations
        .iter()
        .map(|allocation| allocation.sheet_id)
        .collect();
    let mirror_materialized_sheets = allocations
        .get(critical_sheet_index)
        .map(|allocation| {
            std::iter::once(allocation.sheet_id).collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();
    let yrs_hydrated_sheets = mirror_materialized_sheets.clone();
    let sheet_dependency_manifest = SheetDependencyManifest::from_parser_manifest(
        parsed_dependency_manifest,
        &parse_output,
        &allocations,
    )?;
    engine.deferred_hydration = Some(DeferredHydrationData {
        parse_output,
        allocations,
        workbook_snap,
        raw_xlsx_bytes: Some(xlsx_data.to_vec()),
        expected_imported_sheets,
        mirror_materialized_sheets,
        yrs_hydrated_sheets,
        sheet_dependency_manifest,
        committed_cell_edit_batches: Vec::new(),
    });

    Ok(RecalcResult::empty())
}

/// Materialize a single sheet into the engine's preview state without writing
/// it to Yrs or completing the deferred import.
///
/// The retained parse output is cumulative: the critical first-paint sheet and
/// every explicitly requested sheet remain present, while sheets that have not
/// been requested stay metadata-only. Rebuilding from that sparse snapshot is
/// intentional. The existing incremental `add_sheet` path assumes a new sheet
/// and cannot safely replace a metadata-only placeholder with the same ID.
pub(in crate::storage::engine) fn materialize_deferred_sheet(
    engine: &mut YrsComputeEngine,
    sheet_id: SheetId,
) -> Result<(), ComputeError> {
    let Some(mut deferred) = engine.deferred_hydration.take() else {
        return Ok(());
    };

    // Always restore the deferred guard, including on parse/allocation failure.
    // Preview materialization is deliberately not import durability.
    let result = materialize_deferred_sheet_inner(engine, &mut deferred, sheet_id);
    engine.deferred_hydration = Some(deferred);
    result
}

fn materialize_deferred_sheet_inner(
    engine: &mut YrsComputeEngine,
    deferred: &mut DeferredHydrationData,
    sheet_id: SheetId,
) -> Result<(), ComputeError> {
    let sheet_index = deferred
        .allocations
        .iter()
        .position(|allocation| allocation.sheet_id == sheet_id)
        .ok_or_else(|| ComputeError::InvalidInput {
            message: format!("sheet {sheet_id} is not part of the deferred XLSX import"),
        })?;

    // A non-empty snapshot proves this sheet was already parsed. Empty sheets
    // are cheap to re-parse, and callers normally suppress repeat requests via
    // the lifecycle materialization tracker.
    if deferred
        .workbook_snap
        .sheets
        .get(sheet_index)
        .is_some_and(|sheet| !sheet.cells.is_empty() || !sheet.ranges.is_empty())
    {
        deferred.mirror_materialized_sheets.insert(sheet_id);
        return Ok(());
    }

    let raw_bytes =
        deferred
            .raw_xlsx_bytes
            .as_deref()
            .ok_or_else(|| ComputeError::InvalidInput {
                message: "deferred XLSX sheet materialization requires retained source bytes"
                    .into(),
            })?;
    let selected = xlsx_api::parse_selected_sheets(raw_bytes, &[sheet_index]).map_err(|e| {
        ComputeError::Deserialize {
            message: format!("XLSX selected-sheet parse error: {e}"),
        }
    })?;
    let selected_import_report = selected.import_report;
    let selected_sheet = selected
        .output
        .sheets
        .get(sheet_index)
        .cloned()
        .ok_or_else(|| ComputeError::Deserialize {
            message: format!("selected-sheet parse omitted editable sheet index {sheet_index}"),
        })?;

    // Move the retained parse output out while rebuilding it so materializing
    // another sheet does not briefly retain both the old and cumulative copies.
    // The closure boundary restores it on every `?` path below.
    let mut cumulative_parse = std::mem::take(&mut deferred.parse_output);
    let result = (|| -> Result<(), ComputeError> {
        let cumulative_sheet = cumulative_parse
            .sheets
            .get_mut(sheet_index)
            .ok_or_else(|| ComputeError::Deserialize {
                message: format!(
                    "deferred parse output omitted editable sheet index {sheet_index}"
                ),
            })?;
        *cumulative_sheet = selected_sheet;

        use crate::storage::infra::hydration::{
            HydrationIdMap, SharedIdAllocator, allocate_sheet_ids_with_previous_allocation,
        };
        let mut allocator = SharedIdAllocator::from_shared(engine.stores.grid_id_alloc.clone());
        let allocations: Vec<_> = cumulative_parse
            .sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| {
                allocate_sheet_ids_with_previous_allocation(
                    sheet,
                    &mut allocator,
                    deferred.allocations.get(index),
                )
            })
            .collect();

        let mut id_map = HydrationIdMap::default();
        for allocation in &allocations {
            id_map.sheet_ids.push(allocation.sheet_id);
            id_map
                .cell_ids
                .push(std::sync::Arc::clone(&allocation.cell_ids));
            id_map
                .row_ids
                .push(std::sync::Arc::clone(&allocation.row_ids));
            id_map
                .col_ids
                .push(std::sync::Arc::clone(&allocation.col_ids));
            for identity in &allocation.identity_only_cells {
                id_map.identity_only_cells.push((
                    allocation.sheet_id,
                    identity.cell_id,
                    identity.row,
                    identity.col,
                ));
            }
        }

        let cumulative_snap =
            crate::import::parse_output_to_snapshot::parse_output_to_workbook_snapshot(
                &cumulative_parse,
                Some(&id_map),
                &mut allocator,
            );

        // Materialization adds only style projections to the existing Yrs
        // sheet. Values remain snapshot/mirror-backed, and this transaction
        // deliberately does not touch cells or gridIndex.
        let (range_style_positions, imported_range_styles) = if range_style_formats_enabled() {
            build_imported_range_style_plan(
                &cumulative_parse.sheets[sheet_index],
                &allocations[sheet_index],
                &cumulative_snap.sheets[sheet_index].ranges,
                &mut allocator,
            )
        } else {
            (std::collections::HashSet::new(), Vec::new())
        };
        let mut pos_map = std::collections::HashMap::with_capacity(
            cumulative_parse.sheets[sheet_index].cells.len(),
        );
        for (cell_index, cell) in cumulative_parse.sheets[sheet_index]
            .cells
            .iter()
            .enumerate()
        {
            if let Some(cell_id) = allocations[sheet_index].cell_ids.get(cell_index) {
                pos_map.insert(
                    (cell.row, cell.col),
                    compute_document::hex::id_to_hex(cell_id.as_u128()).to_string(),
                );
            }
        }
        {
            // Materialized styles are preview state, not a user mutation.
            // Suppress the normal mutation observer while the style-only
            // transaction writes the existing sheet maps.
            let _guard = engine.mutation.suppress_guard();
            let mut txn = engine.stores.storage.doc().transact_mut();
            let workbook = engine.stores.storage.workbook_map().clone();
            let sheets = engine.stores.storage.sheets().clone();
            let sheet_hex =
                compute_document::hex::id_to_hex(allocations[sheet_index].sheet_id.as_u128());
            let sheet_map = match sheets.get(&txn, &sheet_hex) {
                Some(yrs::Out::YMap(map)) => map,
                _ => {
                    return Err(ComputeError::InvalidInput {
                        message: format!("materialized sheet {sheet_id} is missing from Yrs"),
                    });
                }
            };
            crate::storage::infra::hydration::hydrate_sheet_styles_only(
                &mut txn,
                &workbook,
                &sheet_map,
                &cumulative_parse.sheets[sheet_index],
                &selected.output.style_palette,
                &allocations[sheet_index].row_id_hexes,
                &allocations[sheet_index].col_id_hexes,
                &pos_map,
                &range_style_positions,
                &imported_range_styles,
            );
        }
        // As in commit_deferred_hydration below, pre-full-hydration update
        // bytes are not causally valid for providers. Materialization restores
        // the deferred guard and the read-only path does not drain this
        // buffer, so clear the style transaction's observer payload now.
        engine.update_buffer.clear();

        let shared_alloc = engine.stores.grid_id_alloc.clone();
        let target_range = sheet_index..sheet_index.saturating_add(1);
        let mut target_grid_indexes = build_grid_indexes_from_allocations_range(
            &cumulative_snap,
            &allocations,
            target_range.clone(),
            shared_alloc.clone(),
        )?;
        let target_grid_for_layout = target_grid_indexes.clone();
        let target_grid =
            target_grid_indexes
                .remove(&sheet_id)
                .ok_or_else(|| ComputeError::Deserialize {
                    message: format!("target sheet {sheet_id} grid index was not built"),
                })?;
        let target_merge_indexes = build_merge_indexes_from_parse_output_range(
            &cumulative_parse,
            &cumulative_snap,
            target_range.clone(),
        )?;
        let mut target_layout_indexes = build_layout_indexes_from_parse_output_range(
            &cumulative_parse,
            &cumulative_snap,
            &target_grid_for_layout,
            target_range,
            engine.stores.layout_metrics,
        )?;
        engine
            .stores
            .dimension_preview
            .replay_into(&mut target_layout_indexes);

        let mut compute = ComputeCore::new();
        // The viewport-only initializer intentionally rebuilds workbook-wide
        // formula metadata. Keep that compute handling unchanged for now, but
        // do not let its required temporary mirror replace the live mirror.
        let mut compute_mirror = CellMirror::new();
        compute.init_from_snapshot_viewport_only(&mut compute_mirror, &cumulative_snap)?;
        compute.set_id_alloc(shared_alloc.clone());
        engine
            .mirror
            .replace_sheet(cumulative_snap.sheets[sheet_index].clone())?;
        engine.mirror.replace_row_col_indexes_for_sheet(
            sheet_id,
            target_grid.row_ids_ordered(),
            target_grid.col_ids_ordered(),
        );
        engine.mirror.finalize_range_hydration_for_sheet(&sheet_id);
        if let Some(sheet_mirror) = engine.mirror.get_sheet_mut(&sheet_id) {
            crate::storage::properties::hydrate_col_format_ranges(
                &engine.stores.storage,
                &sheet_id,
                sheet_mirror,
            );
            crate::storage::properties::hydrate_format_ranges(
                &engine.stores.storage,
                &sheet_id,
                sheet_mirror,
            );
        }

        engine.stores.compute = compute;
        engine.stores.grid_id_alloc = shared_alloc;
        engine.stores.grid_indexes.insert(sheet_id, target_grid);
        engine
            .stores
            .merge_indexes
            .insert(sheet_id, target_merge_indexes.into_iter().next().unwrap().1);
        engine.stores.layout_indexes.insert(
            sheet_id,
            target_layout_indexes.into_iter().next().unwrap().1,
        );
        engine.viewport.clear();

        merge_import_reports(&mut engine.import_report, selected_import_report);
        deferred.allocations = allocations;
        deferred.workbook_snap = cumulative_snap;
        deferred.mirror_materialized_sheets.insert(sheet_id);
        Ok(())
    })();

    // Preserve the cumulative parse output even when rebuilding the engine
    // fails partway through, so a later materialization can retry safely.
    deferred.parse_output = cumulative_parse;
    result
}

/// Hydrate one existing deferred sheet into the live Yrs document.
///
/// This is the durable counterpart to [`materialize_deferred_sheet`]. Preview
/// remains style-only; this path replaces exactly one existing sheet subtree,
/// refreshes only that sheet's runtime indexes, and marks the Yrs set only
/// after the bootstrap transaction has committed.
pub(in crate::storage::engine) fn hydrate_deferred_sheet(
    engine: &mut YrsComputeEngine,
    sheet_id: SheetId,
) -> Result<(), ComputeError> {
    let Some(mut deferred) = engine.deferred_hydration.take() else {
        return Ok(());
    };

    let result = hydrate_deferred_sheet_inner(engine, &mut deferred, sheet_id);
    engine.deferred_hydration = Some(deferred);
    result
}

fn hydrate_deferred_sheet_inner(
    engine: &mut YrsComputeEngine,
    deferred: &mut DeferredHydrationData,
    sheet_id: SheetId,
) -> Result<(), ComputeError> {
    if deferred.sheet_is_yrs_hydrated(&sheet_id) {
        // The durable state is monotonic. Repeating a successful request must
        // not allocate new identities, emit another transaction, or touch the
        // existing subtree.
        return Ok(());
    }

    let sheet_index = deferred
        .allocations
        .iter()
        .position(|allocation| allocation.sheet_id == sheet_id)
        .ok_or_else(|| ComputeError::InvalidInput {
            message: format!("sheet {sheet_id} is not part of the deferred XLSX import"),
        })?;

    // Parse only the requested worksheet when it has not already been loaded
    // by the read-only preview path. Keep all cumulative state local until the
    // Yrs transaction and target runtime replacement have succeeded.
    let mut cumulative_parse = deferred.parse_output.clone();
    let mut selected_import_report = None;
    if !deferred.sheet_is_mirror_materialized(&sheet_id) {
        let raw_bytes =
            deferred
                .raw_xlsx_bytes
                .as_deref()
                .ok_or_else(|| ComputeError::InvalidInput {
                    message: "deferred XLSX sheet hydration requires retained source bytes".into(),
                })?;
        let selected = xlsx_api::parse_selected_sheets(raw_bytes, &[sheet_index]).map_err(|e| {
            ComputeError::Deserialize {
                message: format!("XLSX selected-sheet parse error: {e}"),
            }
        })?;
        let selected_sheet = selected
            .output
            .sheets
            .get(sheet_index)
            .cloned()
            .ok_or_else(|| ComputeError::Deserialize {
                message: format!("selected-sheet parse omitted editable sheet index {sheet_index}"),
            })?;
        cumulative_parse.sheets[sheet_index] = selected_sheet;
        selected_import_report = Some(selected.import_report);
    }

    use crate::storage::infra::hydration::{
        HydrationIdMap, SharedIdAllocator, allocate_sheet_ids_with_previous_allocation,
    };
    let mut allocator = SharedIdAllocator::from_shared(engine.stores.grid_id_alloc.clone());
    let mut allocations = deferred.allocations.clone();
    allocations[sheet_index] = allocate_sheet_ids_with_previous_allocation(
        &cumulative_parse.sheets[sheet_index],
        &mut allocator,
        deferred.allocations.get(sheet_index),
    );

    let mut id_map = HydrationIdMap::default();
    for allocation in &allocations {
        id_map.sheet_ids.push(allocation.sheet_id);
        id_map
            .cell_ids
            .push(std::sync::Arc::clone(&allocation.cell_ids));
        id_map
            .row_ids
            .push(std::sync::Arc::clone(&allocation.row_ids));
        id_map
            .col_ids
            .push(std::sync::Arc::clone(&allocation.col_ids));
        for identity in &allocation.identity_only_cells {
            id_map.identity_only_cells.push((
                allocation.sheet_id,
                identity.cell_id,
                identity.row,
                identity.col,
            ));
        }
    }

    let rebuilt_snapshot = {
        use crate::import;
        import::parse_output_to_snapshot::parse_output_to_workbook_snapshot(
            &cumulative_parse,
            Some(&id_map),
            &mut allocator,
        )
    };
    let target_snapshot = rebuilt_snapshot
        .sheets
        .get(sheet_index)
        .cloned()
        .ok_or_else(|| ComputeError::Deserialize {
            message: format!("snapshot rebuild omitted sheet index {sheet_index}"),
        })?;
    validate_deferred_sheet_snapshot(&target_snapshot, sheet_id)?;

    let mut cumulative_snapshot = deferred.workbook_snap.clone();
    if cumulative_snapshot.sheets.len() != rebuilt_snapshot.sheets.len() {
        return Err(ComputeError::Deserialize {
            message: "deferred snapshot inventory changed during per-sheet hydration".into(),
        });
    }
    cumulative_snapshot.sheets[sheet_index] = target_snapshot.clone();

    let range_plan = build_deferred_critical_sheet_range_plan(
        &cumulative_parse.sheets[sheet_index],
        &target_snapshot,
        &allocations[sheet_index],
        &mut allocator,
    );
    let shared_alloc = engine.stores.grid_id_alloc.clone();

    // Build all fallible target runtime structures before replacing the live
    // Yrs subtree. A malformed target therefore cannot leave a half-replaced
    // sheet behind; failures leave both the old subtree and the hydration set
    // untouched.
    let mut target_grid_indexes = build_grid_indexes_from_allocations_range(
        &cumulative_snapshot,
        &allocations,
        sheet_index..sheet_index.saturating_add(1),
        shared_alloc.clone(),
    )?;
    let target_grid_for_layout = target_grid_indexes.clone();
    let mut target_grid =
        target_grid_indexes
            .remove(&sheet_id)
            .ok_or_else(|| ComputeError::Deserialize {
                message: format!("target sheet {sheet_id} grid index was not built"),
            })?;
    let target_merge_indexes = build_merge_indexes_from_parse_output_range(
        &cumulative_parse,
        &cumulative_snapshot,
        sheet_index..sheet_index.saturating_add(1),
    )?;
    let mut target_layout_indexes = build_layout_indexes_from_parse_output_range(
        &cumulative_parse,
        &cumulative_snapshot,
        &target_grid_for_layout,
        sheet_index..sheet_index.saturating_add(1),
        engine.stores.layout_metrics,
    )?;
    engine
        .stores
        .dimension_preview
        .replay_into(&mut target_layout_indexes);
    let theme = cumulative_parse.theme.as_ref();
    let indexed_colors = cumulative_parse
        .workbook_stylesheet
        .as_ref()
        .and_then(|stylesheet| stylesheet.indexed_colors.as_ref());
    let hydration_id_map = {
        // The update observer is independent of the semantic mutation
        // observer. Tag this one transaction separately and remove only its
        // update payload after commit so a queued user update cannot be lost.
        let previous_subscription = std::mem::replace(
            &mut engine._update_subscription,
            crate::storage::engine::update_buffer::install_observer(
                engine.stores.storage.doc(),
                &engine.update_buffer,
                crate::storage::engine::update_buffer::UpdateSource::FullHydration,
            ),
        );
        drop(previous_subscription);
        let result = {
            let _suppress = engine.mutation.suppress_guard();
            engine.stores.storage.hydrate_existing_sheet_with_ranges(
                &cumulative_parse.sheets[sheet_index],
                &cumulative_parse.style_palette,
                &cumulative_parse.persons,
                theme,
                indexed_colors,
                &allocations[sheet_index],
                &range_plan.ranged_positions,
                &range_plan.range_style_positions,
                &range_plan.ranges,
                &range_plan.range_styles,
                &mut allocator,
            )
        };
        let previous_subscription = std::mem::replace(
            &mut engine._update_subscription,
            crate::storage::engine::update_buffer::install_observer(
                engine.stores.storage.doc(),
                &engine.update_buffer,
                crate::storage::engine::update_buffer::UpdateSource::UserMutation,
            ),
        );
        drop(previous_subscription);
        engine
            .update_buffer
            .clear_source(crate::storage::engine::update_buffer::UpdateSource::FullHydration);
        result
    }?;

    for (target_sheet, cell_id, row, col) in &hydration_id_map.phantom_cells {
        if *target_sheet == sheet_id {
            target_grid.register_cell(*cell_id, *row, *col);
        }
    }

    // The target Yrs transaction has committed. The replacement mirror and
    // indexes use the same prevalidated snapshot/allocations and leave every
    // other sheet's runtime state intact.
    engine.mirror.replace_sheet(target_snapshot.clone())?;
    engine.mirror.replace_row_col_indexes_for_sheet(
        sheet_id,
        target_grid.row_ids_ordered(),
        target_grid.col_ids_ordered(),
    );
    engine.mirror.finalize_range_hydration_for_sheet(&sheet_id);
    if let Some(sheet_mirror) = engine.mirror.get_sheet_mut(&sheet_id) {
        crate::storage::properties::hydrate_col_format_ranges(
            &engine.stores.storage,
            &sheet_id,
            sheet_mirror,
        );
        crate::storage::properties::hydrate_format_ranges(
            &engine.stores.storage,
            &sheet_id,
            sheet_mirror,
        );
    }

    engine.stores.grid_indexes.insert(sheet_id, target_grid);
    engine
        .stores
        .merge_indexes
        .insert(sheet_id, target_merge_indexes.into_iter().next().unwrap().1);
    engine.stores.layout_indexes.insert(
        sheet_id,
        target_layout_indexes.into_iter().next().unwrap().1,
    );

    if let Some(selected_import_report) = selected_import_report {
        merge_import_reports(&mut engine.import_report, selected_import_report);
    }
    deferred.parse_output = cumulative_parse;
    deferred.workbook_snap = cumulative_snapshot;
    deferred.allocations = allocations;
    deferred.mirror_materialized_sheets.insert(sheet_id);
    deferred.yrs_hydrated_sheets.insert(sheet_id);

    Ok(())
}

fn validate_deferred_sheet_snapshot(
    snapshot: &SheetSnapshot,
    expected_sheet_id: SheetId,
) -> Result<(), ComputeError> {
    let actual_sheet_id = SheetId::from_uuid_str(&snapshot.id)?;
    if actual_sheet_id != expected_sheet_id {
        return Err(ComputeError::Deserialize {
            message: format!(
                "deferred sheet snapshot identity changed from {expected_sheet_id} to {actual_sheet_id}"
            ),
        });
    }
    for cell in &snapshot.cells {
        CellId::from_uuid_str(&cell.cell_id)?;
    }
    Ok(())
}

/// Complete the deferred Yrs CRDT hydration.
/// Call after first viewport paint to enable mutations and persistence.
pub(in crate::storage::engine) fn stage_deferred_hydration(
    engine: &YrsComputeEngine,
) -> Result<Option<DeferredHydrationCompletion>, ComputeError> {
    let Some(data) = engine.deferred_hydration.as_ref() else {
        return Ok(None);
    };

    // Debug breadcrumbs for WASM std::time panic investigation.
    // tracing::info! routes through the configured subscriber → browser console.
    macro_rules! dh_log {
        ($msg:expr) => {
            tracing::info!(target: "deferred_hydration", $msg);
        };
    }

    let completion = {
        // Pass 0: Re-parse XLSX with full options (all sheets' cells).
        // The fast path only parsed the first sheet's cells. Keep the pending
        // guard installed until every fallible full-hydration step has staged
        // successfully; a failed hydrate must remain retryable/protected.
        dh_log!("phase 0: re-parse XLSX");
        let (full_parse_output, mut import_report) = {
            let mut profile =
                crate::xlsx_profile::PhaseTimer::new("complete_deferred_hydration", "parse");
            let (parsed, report) = if let Some(raw_bytes) = &data.raw_xlsx_bytes {
                let parsed = xlsx_api::parse(raw_bytes).map_err(|e| ComputeError::Deserialize {
                    message: format!("XLSX full re-parse error: {}", e),
                })?;
                (parsed.output, parsed.import_report)
            } else {
                (data.parse_output.clone(), engine.import_report.clone())
            };
            profile.counter("sheets", parsed.sheets.len() as u64);
            profile.counter(
                "cells",
                parsed
                    .sheets
                    .iter()
                    .map(|sheet| sheet.cells.len() as u64)
                    .sum::<u64>(),
            );
            let mut merged_report = engine.import_report.clone();
            merge_import_reports(&mut merged_report, report);
            (parsed, merged_report)
        };
        dh_log!("phase 0 done");

        // Pass 1: Re-allocate IDs for ALL sheets, reusing any IDs assigned by
        // the first-paint parse while still consuming allocator slots for the
        // full workbook shape.
        use crate::storage::infra::hydration::{
            SharedIdAllocator, allocate_sheet_ids_with_previous_allocation,
        };
        let mut allocator = SharedIdAllocator::from_shared(engine.stores.grid_id_alloc.clone());
        let allocations: Vec<_> = {
            let mut profile = crate::xlsx_profile::PhaseTimer::new(
                "complete_deferred_hydration",
                "id_allocation",
            );
            let allocations: Vec<_> = full_parse_output
                .sheets
                .iter()
                .enumerate()
                .map(|(i, sheet)| {
                    allocate_sheet_ids_with_previous_allocation(
                        sheet,
                        &mut allocator,
                        data.allocations.get(i),
                    )
                })
                .collect();
            profile.counter("sheets", allocations.len() as u64);
            profile.counter(
                "allocated_cells",
                allocations
                    .iter()
                    .map(|allocation| allocation.cell_ids.len() as u64)
                    .sum::<u64>(),
            );
            profile.counter(
                "identity_rows",
                allocations
                    .iter()
                    .map(|allocation| allocation.row_ids.len() as u64)
                    .sum::<u64>(),
            );
            profile.counter(
                "identity_cols",
                allocations
                    .iter()
                    .map(|allocation| allocation.col_ids.len() as u64)
                    .sum::<u64>(),
            );
            allocations
        };

        let id_map_full = {
            use crate::storage::infra::hydration::HydrationIdMap;
            let mut m = HydrationIdMap::default();
            for alloc in &allocations {
                m.sheet_ids.push(alloc.sheet_id);
                m.cell_ids.push(std::sync::Arc::clone(&alloc.cell_ids));
                m.row_ids.push(std::sync::Arc::clone(&alloc.row_ids));
                m.col_ids.push(std::sync::Arc::clone(&alloc.col_ids));
                for identity in &alloc.identity_only_cells {
                    m.identity_only_cells.push((
                        alloc.sheet_id,
                        identity.cell_id,
                        identity.row,
                        identity.col,
                    ));
                }
            }
            m
        };

        let full_snap = {
            use crate::import;
            let mut profile = crate::xlsx_profile::PhaseTimer::new(
                "complete_deferred_hydration",
                "parse_output_to_workbook_snapshot",
            );
            let snap = import::parse_output_to_snapshot::parse_output_to_workbook_snapshot(
                &full_parse_output,
                Some(&id_map_full),
                &mut allocator,
            );
            profile.counter("sheets", snap.sheets.len() as u64);
            profile.counter(
                "snapshot_cells",
                snap.sheets
                    .iter()
                    .map(|sheet| sheet.cells.len() as u64)
                    .sum::<u64>(),
            );
            profile.counter(
                "ranges",
                snap.sheets
                    .iter()
                    .map(|sheet| sheet.ranges.len() as u64)
                    .sum::<u64>(),
            );
            snap
        };

        let mut ranged_positions: Vec<std::collections::HashSet<(u32, u32)>> =
            Vec::with_capacity(full_parse_output.sheets.len());
        let mut range_data_per_sheet: Vec<&[snapshot_types::RangeData]> =
            Vec::with_capacity(full_parse_output.sheets.len());
        let mut range_style_positions: Vec<std::collections::HashSet<(u32, u32)>> =
            Vec::with_capacity(full_parse_output.sheets.len());
        let mut range_styles_per_sheet: Vec<
            Vec<crate::storage::infra::hydration::ImportedRangeStyle>,
        > = Vec::with_capacity(full_parse_output.sheets.len());
        {
            let mut profile = crate::xlsx_profile::PhaseTimer::new(
                "complete_deferred_hydration",
                "ranged_positions",
            );
            for (sheet_idx, sheet_data) in full_parse_output.sheets.iter().enumerate() {
                let snap_sheet = &full_snap.sheets[sheet_idx];
                let snap_positions: std::collections::HashSet<(u32, u32)> =
                    snap_sheet.cells.iter().map(|c| (c.row, c.col)).collect();
                let ranged: std::collections::HashSet<(u32, u32)> = sheet_data
                    .cells
                    .iter()
                    .filter(|c| c.formula().is_some() || !c.value.is_null())
                    .map(|c| (c.row, c.col))
                    .filter(|pos| !snap_positions.contains(pos))
                    .collect();
                ranged_positions.push(ranged);
                range_data_per_sheet.push(snap_sheet.ranges.as_slice());
                range_style_positions.push(std::collections::HashSet::new());
                range_styles_per_sheet.push(Vec::new());
            }
            profile.counter("sheets", ranged_positions.len() as u64);
            profile.counter(
                "ranged_positions",
                ranged_positions
                    .iter()
                    .map(|positions| positions.len() as u64)
                    .sum::<u64>(),
            );
        }

        dh_log!("phase 1 done: IDs allocated, snapshot built");

        dh_log!("phase 2a: YrsStorage::new()");
        let mut new_storage = YrsStorage::new();
        dh_log!("phase 2b: hydrate_from_parse_output_with_ranges start");
        let mut profile = crate::xlsx_profile::PhaseTimer::new(
            "complete_deferred_hydration",
            "hydrate_from_parse_output_with_ranges",
        );
        let id_map = new_storage.hydrate_from_parse_output_with_ranges(
            &full_parse_output,
            &allocations,
            &ranged_positions,
            &range_style_positions,
            &range_data_per_sheet,
            &range_styles_per_sheet,
            &mut allocator,
        )?;
        new_storage.hydrate_imported_external_links(&full_parse_output.external_links)?;
        profile.counter("sheets", full_parse_output.sheets.len() as u64);
        profile.counter(
            "ranged_positions",
            ranged_positions
                .iter()
                .map(|positions| positions.len() as u64)
                .sum::<u64>(),
        );

        dh_log!("phase 2 done: YrsStorage hydrated");

        let shared_alloc = engine.stores.grid_id_alloc.clone();
        let layout_metrics = engine.stores.layout_metrics;
        let grid_indexes =
            build_grid_indexes_from_yrs(&new_storage, &full_snap, shared_alloc.clone())?;
        let merge_indexes = build_merge_indexes(&new_storage, &full_snap, &grid_indexes)?;
        let layout_indexes =
            build_layout_indexes(&new_storage, &full_snap, &grid_indexes, layout_metrics)?;

        dh_log!("phase 3 done: grid/merge/layout indexes built");

        // Rebuild ComputeCore and CellMirror against the staged full snapshot
        // before committing them to the live engine.
        let (new_compute, mut new_mirror) = {
            let mut profile = crate::xlsx_profile::PhaseTimer::new(
                "complete_deferred_hydration",
                "mirror_compute_rebuild",
            );
            let snapshot_sheet_count = full_snap.sheets.len() as u64;
            let snapshot_cell_count = full_snap
                .sheets
                .iter()
                .map(|sheet| sheet.cells.len() as u64)
                .sum::<u64>();
            profile.counter("sheets", snapshot_sheet_count);
            profile.counter("snapshot_cells", snapshot_cell_count);
            let mut new_compute = ComputeCore::new();
            let mut new_mirror = CellMirror::new();
            #[cfg(target_arch = "wasm32")]
            {
                new_compute.init_from_snapshot_minimal(&mut new_mirror, full_snap)?;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                new_compute.init_from_snapshot_no_recalc(&mut new_mirror, full_snap)?;
            }
            new_compute.set_id_alloc(shared_alloc.clone());
            (new_compute, new_mirror)
        };

        dh_log!("phase 4 done: ComputeCore init_from_snapshot_minimal");

        new_mirror.install_row_col_indexes(
            grid_indexes
                .iter()
                .map(|(sid, grid)| (*sid, grid.row_ids_ordered(), grid.col_ids_ordered())),
        );
        hydrate_mirror_format_ranges(&new_storage, &mut new_mirror);
        new_mirror.finalize_range_hydration();

        let settings = derive_settings(&new_storage);
        let calculation = full_parse_output.calculation.clone();
        if !full_parse_output.sheets.is_empty() && new_storage.sheet_order().is_empty() {
            return Err(ComputeError::InvalidInput {
                message: "deferred hydration produced an empty Yrs document; refusing to commit (would clear the export guard over nothing)".into(),
            });
        }
        let id_alloc = std::sync::Arc::new(crate::storage::metadata_id_allocator_for_doc_client(
            new_storage.doc().client_id(),
        ));
        let mut stores = EngineStores {
            storage: new_storage,
            layout_metrics,
            grid_id_alloc: shared_alloc,
            id_alloc,
            grid_indexes,
            layout_indexes,
            dimension_preview: Default::default(),
            merge_indexes,
            compute: new_compute,
            cf_cache: FxHashMap::default(),
            font_db: compute_text_measurement::FontDb::with_defaults(),
            measurement_cache: compute_text_measurement::MeasurementCache::new(),
            custom_table_styles: FxHashMap::default(),
            custom_cell_styles: FxHashMap::default(),
        };

        load_custom_cell_styles(&mut stores);
        load_custom_table_styles(&mut stores);
        sync_table_catalog_from_yrs_if_present(&mut stores, &mut new_mirror);
        crate::storage::engine::services::imported_filters::normalize_imported_auto_filter_visibility(
            &mut stores,
            &mut new_mirror,
            Some(&mut import_report),
            domain_types::ImportPhase::FullHydration,
        );

        DeferredHydrationCompletion {
            stores,
            mirror: new_mirror,
            settings,
            phantom_cells: id_map.phantom_cells,
            calculation,
            import_report,
            committed_cell_edit_batches: data.committed_cell_edit_batches.clone(),
        }
    };

    dh_log!("phase 5 done: mirror finalized, settings derived");

    Ok(Some(completion))
}

fn merge_import_reports(
    target: &mut domain_types::ImportReport,
    source: domain_types::ImportReport,
) {
    target.diagnostics.extend(source.diagnostics);
    target.force_recalc_cells.extend(source.force_recalc_cells);
    target.object_statuses.extend(source.object_statuses);
    target.stats = source.stats;
    target.canonicalize();
}

#[derive(Default)]
struct DeferredCriticalSheetRangePlan {
    ranged_positions: std::collections::HashSet<(u32, u32)>,
    ranges: Vec<snapshot_types::RangeData>,
    range_style_positions: std::collections::HashSet<(u32, u32)>,
    range_styles: Vec<crate::storage::infra::hydration::ImportedRangeStyle>,
}

fn build_deferred_critical_sheet_range_plan(
    sheet_data: &domain_types::SheetData,
    snap_sheet: &SheetSnapshot,
    allocation: &crate::storage::infra::hydration::SheetIdAllocation,
    allocator: &mut crate::storage::infra::hydration::SharedIdAllocator,
) -> DeferredCriticalSheetRangePlan {
    let snap_positions: std::collections::HashSet<(u32, u32)> =
        snap_sheet.cells.iter().map(|c| (c.row, c.col)).collect();
    let ranged_positions = sheet_data
        .cells
        .iter()
        .filter(|c| c.formula().is_some() || !c.value.is_null())
        .map(|c| (c.row, c.col))
        .filter(|pos| !snap_positions.contains(pos))
        .collect();
    let ranges = snap_sheet.ranges.clone();
    let (range_style_positions, range_styles) = range_style_formats_enabled()
        .then(|| build_imported_range_style_plan(sheet_data, allocation, &ranges, allocator))
        .unwrap_or_default();

    DeferredCriticalSheetRangePlan {
        ranged_positions,
        ranges,
        range_style_positions,
        range_styles,
    }
}

pub(in crate::storage::engine) fn commit_deferred_hydration(
    engine: &mut YrsComputeEngine,
    completion: DeferredHydrationCompletion,
) -> Result<(), ComputeError> {
    let committed_cell_edit_batches = completion.committed_cell_edit_batches;
    // Commit the fully staged state. From this point onward the live engine is
    // all-sheet materialized and export/graph guards can be cleared.
    engine.update_buffer.clear();
    engine.stores = completion.stores;
    engine._update_subscription = crate::storage::engine::update_buffer::install_observer(
        engine.stores.storage.doc(),
        &engine.update_buffer,
        crate::storage::engine::update_buffer::UpdateSource::FullHydration,
    );
    engine.mirror = completion.mirror;

    let (observer, undo_manager) = create_observer_and_undo(&engine.stores.storage);
    engine.mutation.observer = observer;
    engine.mutation.undo_manager = undo_manager;
    engine.settings = completion.settings;
    engine.import_report = completion.import_report;
    engine.viewport.clear();

    engine.init_cf_caches();

    normalize_named_range_refs(engine);
    sync_enable_calculation_flags(engine);

    for (sheet_id, cell_id, row, col) in completion.phantom_cells {
        if let Some(grid) = engine.stores.grid_indexes.get_mut(&sheet_id) {
            grid.register_cell(cell_id, row, col);
        }
    }

    engine.deferred_hydration = None;

    // Full hydration and its normalization writes are bootstrap state. Clear
    // only those updates, then restore the normal user source before replaying
    // row-4 edits so their undo and provider semantics match the original
    // admitted mutations.
    engine
        .update_buffer
        .clear_source(crate::storage::engine::update_buffer::UpdateSource::FullHydration);
    let previous_subscription = std::mem::replace(
        &mut engine._update_subscription,
        crate::storage::engine::update_buffer::install_observer(
            engine.stores.storage.doc(),
            &engine.update_buffer,
            crate::storage::engine::update_buffer::UpdateSource::UserMutation,
        ),
    );
    drop(previous_subscription);

    for batch in committed_cell_edit_batches {
        let mutation = match batch.kind {
            DeferredCommittedCellEditKind::SetCell => {
                let edit =
                    batch
                        .edits
                        .into_iter()
                        .next()
                        .ok_or_else(|| ComputeError::InvalidInput {
                            message: "deferred SetCell replay batch was empty".into(),
                        })?;
                crate::storage::engine::mutation::EngineMutation::SetCell {
                    sheet_id: edit.sheet_id,
                    cell_id: edit.cell_id,
                    row: edit.row,
                    col: edit.col,
                    input: edit.input,
                }
            }
            DeferredCommittedCellEditKind::SetCells => {
                crate::storage::engine::mutation::EngineMutation::SetCells {
                    edits: batch
                        .edits
                        .into_iter()
                        .map(|edit| (edit.sheet_id, edit.cell_id, edit.row, edit.col, edit.input))
                        .collect(),
                    skip_cycle_check: batch.skip_cycle_check,
                }
            }
            DeferredCommittedCellEditKind::SetCellsByPosition => {
                // Preserve identities minted by the original position edit.
                // Once registered, normal position resolution reuses them and
                // still executes the format-inference path for this variant.
                for edit in &batch.edits {
                    engine
                        .stores
                        .grid_indexes
                        .get_mut(&edit.sheet_id)
                        .ok_or_else(|| ComputeError::InvalidInput {
                            message: format!(
                                "deferred edit replay is missing sheet {} grid index",
                                edit.sheet_id
                            ),
                        })?
                        .register_cell(edit.cell_id, edit.row, edit.col);
                }
                crate::storage::engine::mutation::EngineMutation::SetCellsByPosition {
                    edits: batch
                        .edits
                        .into_iter()
                        .map(|edit| (edit.sheet_id, edit.row, edit.col, edit.input))
                        .collect(),
                    skip_cycle_check: batch.skip_cycle_check,
                }
            }
        };
        engine.apply_mutation(mutation)?;
    }

    Ok(())
}
