use super::support::*;
use super::*;

use cell_types::{CellId, SheetId};
use domain_types::{
    Comment, CommentType, DocumentFormat, FillFormat, SheetData, TableColumnSpec, TableSpec,
};
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use value_types::CellValue;

fn imported_sheet_ids(engine: &YrsComputeEngine) -> Vec<SheetId> {
    engine
        .get_all_sheet_ids()
        .iter()
        .map(|id| SheetId::from_uuid_str(id).expect("imported sheet id should parse"))
        .collect()
}

fn domain_fixture_xlsx() -> Vec<u8> {
    let output = domain_types::ParseOutput {
        style_palette: vec![
            DocumentFormat::default(),
            DocumentFormat {
                number_format: Some("$#,##0.00".to_string()),
                fill: Some(FillFormat {
                    background_color: Some("#00CC99".to_string()),
                    pattern_type: Some("solid".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        sheets: vec![
            SheetData {
                name: "Landing".to_string(),
                rows: 1,
                cols: 1,
                cells: vec![domain_types::CellData {
                    row: 0,
                    col: 0,
                    value: CellValue::number(1.0),
                    ..Default::default()
                }],
                tables: vec![TableSpec {
                    id: 1,
                    name: "LandingTable".to_string(),
                    display_name: "LandingTable".to_string(),
                    range_ref: "A1:A1".to_string(),
                    has_headers: true,
                    has_totals: false,
                    style_name: Some("TableStyleMedium2".to_string()),
                    row_stripes: true,
                    auto_filter_ref: Some("A1:A1".to_string()),
                    columns: vec![TableColumnSpec {
                        name: "Value".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            SheetData {
                name: "Domain".to_string(),
                rows: 3,
                cols: 2,
                cells: vec![
                    domain_types::CellData {
                        row: 0,
                        col: 0,
                        value: CellValue::Text("Name".into()),
                        style_id: Some(1),
                        ..Default::default()
                    },
                    domain_types::CellData {
                        row: 0,
                        col: 1,
                        value: CellValue::Text("Score".into()),
                        style_id: Some(1),
                        ..Default::default()
                    },
                    domain_types::CellData {
                        row: 1,
                        col: 0,
                        value: CellValue::Text("Ada".into()),
                        ..Default::default()
                    },
                    domain_types::CellData {
                        row: 1,
                        col: 1,
                        value: CellValue::number(95.0),
                        ..Default::default()
                    },
                    domain_types::CellData {
                        row: 2,
                        col: 0,
                        value: CellValue::Text("Grace".into()),
                        ..Default::default()
                    },
                    domain_types::CellData {
                        row: 2,
                        col: 1,
                        value: CellValue::number(88.0),
                        ..Default::default()
                    },
                ],
                comments: vec![Comment {
                    cell_ref: "A1".to_string(),
                    author: "Tester".to_string(),
                    content: Some("Header comment".to_string()),
                    comment_type: CommentType::Note,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    xlsx_parser::write::write_xlsx_from_parse_output(&output)
        .expect("domain preservation fixture should be writable")
}

fn range_backed_fixture_xlsx() -> Vec<u8> {
    let mut range_backed_cells = Vec::with_capacity(3823);
    for row in 0..3822 {
        range_backed_cells.push(domain_types::CellData {
            row,
            col: 16,
            value: CellValue::number(34532.0 + row as f64),
            ..Default::default()
        });
    }
    range_backed_cells.push(domain_types::CellData {
        row: 3821,
        col: 29,
        value: CellValue::number(2005.0),
        formula: Some(r#"IF(Q3822="","",YEAR(Q3822))"#.to_string()),
        ..Default::default()
    });

    let output = domain_types::ParseOutput {
        sheets: vec![
            SheetData {
                name: "Landing".to_string(),
                rows: 2,
                cols: 1,
                cells: vec![domain_types::CellData {
                    row: 0,
                    col: 0,
                    value: CellValue::number(1.0),
                    ..Default::default()
                }],
                ..Default::default()
            },
            SheetData {
                name: "RangeBacked".to_string(),
                rows: 3822,
                cols: 30,
                cells: range_backed_cells,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    xlsx_parser::write::write_xlsx_from_parse_output(&output)
        .expect("range preservation fixture should be writable")
}

#[test]
fn existing_sheet_hydration_uses_one_transaction_and_keeps_sheet_order() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let ids = imported_sheet_ids(&engine);
    let target = ids[1];
    let later = ids[2];
    let order_before = engine.storage().sheet_order();
    let transactions = Arc::new(AtomicUsize::new(0));
    let transactions_for_observer = Arc::clone(&transactions);
    let _subscription = compute_collab::subscribe_update_v1(engine.storage().doc(), move |_| {
        transactions_for_observer.fetch_add(1, Ordering::SeqCst);
    });

    engine
        .hydrate_deferred_sheet(target)
        .expect("target sheet hydration should succeed");

    assert_eq!(
        transactions.load(Ordering::SeqCst),
        1,
        "one target-sheet hydration must commit exactly one Yrs transaction"
    );
    assert_eq!(engine.storage().sheet_order(), order_before);
    assert_eq!(
        engine.get_cell_value(&target, 0, 0),
        CellValue::number(579.0)
    );
    assert!(engine.get_cell_id_at(&target, 0, 0).is_some());
    assert!(engine.get_cell_id_at_yrs(&target, 0, 0).is_some());
    assert!(
        engine.get_cell_id_at(&later, 0, 0).is_none(),
        "a later sheet must remain mirror-unmaterialized"
    );
    assert!(
        engine.get_cell_id_at_yrs(&later, 0, 0).is_none(),
        "hydrating one sheet must not populate another sheet's cells"
    );
    assert!(
        engine
            .drain_pending_updates()
            .expect("bootstrap update drain should succeed")
            .is_empty(),
        "the bootstrap transaction must not reach the provider queue"
    );

    engine
        .hydrate_deferred_sheet(later)
        .expect("later sheet hydration should succeed");
    assert_eq!(
        transactions.load(Ordering::SeqCst),
        2,
        "each newly hydrated sheet must contribute exactly one transaction"
    );
    assert_eq!(engine.storage().sheet_order(), order_before);
}

#[test]
fn existing_sheet_hydration_preserves_user_edit_and_pending_update() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let ids = imported_sheet_ids(&engine);
    let landing = ids[0];
    let target = ids[1];
    let landing_a1_before = engine
        .get_cell_id_at(&landing, 0, 0)
        .expect("landing A1 should have a stable mirror identity");
    let edited_cell_id = CellId::from_uuid_str(&landing_a1_before)
        .expect("landing A1 should have a parseable CellId");
    engine.with_storage_and_mirror_for_test(|storage, mirror| {
        storage.set_cell(
            mirror,
            &landing,
            edited_cell_id,
            0,
            0,
            CellValue::Text("user edit".into()),
            None,
            None,
        );
    });
    let pending_before = engine.update_buffer.len();
    assert!(pending_before > 0, "the user edit should be queued");
    let edited_cell_id_string = edited_cell_id.to_uuid_string();
    assert_eq!(
        engine.get_cell_id_at(&landing, 0, 0).as_deref(),
        Some(edited_cell_id_string.as_str()),
        "the low-level admitted edit should register its allocated identity"
    );

    engine
        .hydrate_deferred_sheet(target)
        .expect("target sheet hydration should succeed");

    assert_eq!(
        engine.get_cell_value(&landing, 0, 0),
        CellValue::Text("user edit".into())
    );
    assert_eq!(
        engine.get_cell_id_at(&landing, 0, 0).as_deref(),
        Some(landing_a1_before.as_str())
    );
    assert_eq!(
        engine.get_cell_id_at_yrs(&landing, 0, 0).as_deref(),
        Some(edited_cell_id_string.as_str()),
        "hydrating another sheet must not replace the landing-sheet Yrs edit"
    );
    assert_eq!(
        engine.update_buffer.len(),
        pending_before,
        "clearing FullHydration bytes must preserve an already queued user update"
    );
    assert!(
        !engine
            .drain_pending_updates()
            .expect("user update drain should succeed")
            .is_empty(),
        "the user edit must remain provider-visible after target hydration"
    );
}

#[test]
fn existing_sheet_hydration_preserves_styles_comments_and_tables() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&domain_fixture_xlsx())
        .expect("deferred domain import should succeed");

    let ids = imported_sheet_ids(&engine);
    let landing = ids[0];
    let target = ids[1];
    let comments_before = engine.get_all_comments(&target);
    let tables_before = engine.get_all_tables_in_sheet(&landing);
    assert!(
        comments_before.is_empty(),
        "comments on a nonlanding sheet remain parse-only until durable hydration"
    );
    assert_eq!(
        tables_before.len(),
        1,
        "the landing table should hydrate at bootstrap"
    );

    engine
        .hydrate_deferred_sheet(target)
        .expect("domain sheet hydration should succeed");

    let target_a1 = engine
        .get_cell_id_at(&target, 0, 0)
        .expect("hydrated target A1 should have a mirror identity");
    let format = engine.get_cell_format(
        &target,
        &cell_types::CellId::from_uuid_str(&target_a1).expect("target A1 id"),
        0,
        0,
    );
    assert_eq!(format.number_format.as_deref(), Some("$#,##0.00"));
    assert!(
        format.pattern_foreground_color.as_deref() == Some("#00CC99")
            || format.background_color.as_deref() == Some("#00CC99"),
        "target style must survive subtree replacement: {format:?}"
    );
    let comments_after = engine.get_all_comments(&target);
    assert_eq!(
        comments_after.len(),
        1,
        "the target note should hydrate into Yrs"
    );
    assert_eq!(
        comments_after[0].content.as_deref(),
        Some("Tester:\nHeader comment")
    );
    let tables_after = engine.get_all_tables_in_sheet(&landing);
    assert_eq!(tables_after, tables_before);
    assert_eq!(tables_after[0].name, "LandingTable");
}

#[test]
fn existing_sheet_hydration_preserves_range_data() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&range_backed_fixture_xlsx())
        .expect("deferred range import should succeed");

    let target = imported_sheet_ids(&engine)[1];
    assert!(engine.get_cell_id_at(&target, 3821, 16).is_none());

    engine
        .hydrate_deferred_sheet(target)
        .expect("range-backed target hydration should succeed");

    assert_eq!(
        engine.get_cell_value(&target, 3821, 16),
        CellValue::number(38353.0)
    );
    let sheet = engine
        .mirror()
        .get_sheet(&target)
        .expect("range-backed target mirror should exist");
    assert!(
        sheet.iter_ranges().count() > 0,
        "existing-sheet hydration must write and retain RangeData"
    );
}

#[test]
fn malformed_existing_sheet_allocation_is_atomic_and_retryable() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let target_index = 1;
    let (target_sheet, mut malformed_allocation, yrs_before, mirror_before) = {
        let state = engine
            .deferred_hydration
            .as_ref()
            .expect("deferred state should exist");
        (
            state.parse_output.sheets[target_index].clone(),
            state.allocations[target_index].clone(),
            state.yrs_hydrated_sheets.clone(),
            state.mirror_materialized_sheets.clone(),
        )
    };
    let target_id = malformed_allocation.sheet_id;
    malformed_allocation
        .row_id_hexes
        .push(compute_document::hex::id_to_hex(0));
    let mut cell_ids = malformed_allocation.cell_ids.to_vec();
    cell_ids.push(CellId::from_raw(u128::MAX));
    malformed_allocation.cell_ids = cell_ids.into();
    let yrs_state_before = compute_collab::encode_full_state(engine.storage().doc());
    let pending_before = engine.update_buffer.len();
    let mut allocator = crate::storage::infra::hydration::SharedIdAllocator::from_shared(
        engine.stores.grid_id_alloc.clone(),
    );

    let result = engine.with_storage_and_mirror_for_test(|storage, _| {
        storage.hydrate_existing_sheet_with_ranges(
            &target_sheet,
            &[],
            &[],
            None,
            None,
            &malformed_allocation,
            &HashSet::new(),
            &HashSet::new(),
            Vec::<snapshot_types::RangeData>::new(),
            &[],
            &mut allocator,
        )
    });
    assert!(
        result.is_err(),
        "malformed allocation must fail before Yrs writes"
    );
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_state_before,
        "a failed replacement must leave the target subtree byte-identical"
    );
    let state = engine.deferred_hydration.as_ref().expect("deferred state");
    assert_eq!(state.yrs_hydrated_sheets, yrs_before);
    assert_eq!(state.mirror_materialized_sheets, mirror_before);
    assert_eq!(engine.update_buffer.len(), pending_before);

    engine
        .hydrate_deferred_sheet(target_id)
        .expect("a failed target hydration must remain retryable");
    let state = engine.deferred_hydration.as_ref().expect("deferred state");
    assert!(state.yrs_hydrated_sheets.contains(&target_id));
}

#[test]
fn repeated_existing_sheet_hydration_is_idempotent() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");
    let target = imported_sheet_ids(&engine)[1];
    let transactions = Arc::new(AtomicUsize::new(0));
    let transactions_for_observer = Arc::clone(&transactions);
    let _subscription = compute_collab::subscribe_update_v1(engine.storage().doc(), move |_| {
        transactions_for_observer.fetch_add(1, Ordering::SeqCst);
    });

    engine
        .hydrate_deferred_sheet(target)
        .expect("first target hydration should succeed");
    let state_before = {
        let state = engine.deferred_hydration.as_ref().expect("deferred state");
        (
            state.yrs_hydrated_sheets.clone(),
            state.mirror_materialized_sheets.clone(),
        )
    };
    let yrs_state_before = compute_collab::encode_full_state(engine.storage().doc());
    let pending_before = engine.update_buffer.len();

    engine
        .hydrate_deferred_sheet(target)
        .expect("repeated target hydration should be a no-op");

    assert_eq!(transactions.load(Ordering::SeqCst), 1);
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_state_before
    );
    let state = engine.deferred_hydration.as_ref().expect("deferred state");
    assert_eq!(
        (
            state.yrs_hydrated_sheets.clone(),
            state.mirror_materialized_sheets.clone()
        ),
        state_before
    );
    assert_eq!(engine.update_buffer.len(), pending_before);
}

#[test]
fn preview_materialization_remains_style_only_until_existing_sheet_hydration() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&materialized_style_deferred_fixture_xlsx())
        .expect("deferred style import should succeed");

    let target = imported_sheet_ids(&engine)[1];
    let yrs_cell_before = engine.storage().read_cell_id_at_pos(&target, 0, 0);
    engine
        .materialize_deferred_sheet(target)
        .expect("preview materialization should succeed");
    assert!(engine.get_cell_id_at(&target, 0, 0).is_some());
    assert_eq!(
        engine.storage().read_cell_id_at_pos(&target, 0, 0),
        yrs_cell_before,
        "preview materialization must not change canonical target cells"
    );
    assert!(
        !engine
            .deferred_hydration
            .as_ref()
            .expect("deferred state")
            .sheet_is_yrs_hydrated(&target)
    );

    engine
        .hydrate_deferred_sheet(target)
        .expect("durable target hydration should succeed");
    assert!(engine.get_cell_id_at_yrs(&target, 0, 0).is_some());
    assert!(
        engine
            .storage()
            .read_cell_id_at_pos(&target, 0, 0)
            .is_some()
    );
    assert!(
        engine
            .deferred_hydration
            .as_ref()
            .expect("deferred state")
            .sheet_is_yrs_hydrated(&target)
    );
}
