use super::support::*;
use super::*;
use cell_types::SheetId;
use formula_types::StructureChange;

#[test]
fn deferred_hydration_state_tracks_landing_and_unhydrated_sheets_explicitly() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let ids = engine.get_all_sheet_ids();
    let landing = SheetId::from_uuid_str(&ids[0]).expect("landing sheet id");
    let other = SheetId::from_uuid_str(&ids[1]).expect("other sheet id");
    let state = engine
        .deferred_hydration
        .as_ref()
        .expect("deferred import should retain explicit hydration state");

    assert_eq!(state.expected_imported_sheets.len(), 3);
    assert!(state.sheet_is_mirror_materialized(&landing));
    assert!(state.sheet_is_yrs_hydrated(&landing));
    assert!(!state.sheet_is_mirror_materialized(&other));
    assert!(!state.sheet_is_yrs_hydrated(&other));
    assert!(!state.all_expected_sheets_yrs_hydrated());
}

#[test]
fn deferred_hydration_guard_uses_exact_set_equality() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let expected = engine
        .deferred_hydration
        .as_ref()
        .expect("deferred state")
        .expected_imported_sheets
        .clone();
    let state = engine.deferred_hydration.as_mut().expect("deferred state");
    state.mirror_materialized_sheets = expected.clone();
    state.yrs_hydrated_sheets = expected;

    assert!(
        engine
            .require_deferred_hydration_complete("test export")
            .is_ok(),
        "exact equality must clear the deferred guard even while the payload is retained"
    );
}

#[test]
fn deferred_hydration_guard_rejects_missing_and_inconsistent_sets() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&three_sheet_deferred_fixture_xlsx())
        .expect("deferred XLSX import should succeed");

    let missing_err = engine
        .require_deferred_hydration_complete("test export")
        .expect_err("missing sheets must remain guarded");
    assert!(missing_err.to_string().contains("missing_count=2"));
    assert!(
        missing_err
            .to_string()
            .contains("action=complete_deferred_hydration")
    );

    let landing = SheetId::from_uuid_str(
        &engine
            .get_all_sheet_ids()
            .first()
            .cloned()
            .expect("landing sheet id"),
    )
    .expect("landing sheet id should parse");
    let state = engine.deferred_hydration.as_mut().expect("deferred state");
    state.expected_imported_sheets.clear();
    state.mirror_materialized_sheets.insert(landing);
    state.yrs_hydrated_sheets.clear();

    let inconsistent_err = engine
        .require_deferred_hydration_complete("test export")
        .expect_err("an inconsistent empty inventory must never clear the guard");
    assert!(
        inconsistent_err
            .to_string()
            .contains("inconsistent deferred XLSX hydration state")
    );
}

#[test]
fn empty_yrs_storage_with_nonempty_mirror_rejects_export_and_sync_diff() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine.with_storage_and_mirror_for_test(|_, mirror| {
        *mirror = crate::mirror::CellMirror::from_snapshot(simple_snapshot())
            .expect("mirror fixture should build");
    });

    assert!(engine.storage().sheet_order().is_empty());
    assert_eq!(engine.mirror().sheet_count(), 1);

    let export_err = engine
        .export_to_parse_output()
        .expect_err("export must reject an empty Yrs document with mirrored sheets");
    assert!(
        export_err.to_string().contains("empty"),
        "export guard should name the empty Yrs state: {export_err}"
    );

    let remote_state_vector = engine.encode_state_vector();
    let diff_err = engine
        .encode_diff(&remote_state_vector)
        .expect_err("sync diff must reject an empty Yrs document with mirrored sheets");
    assert!(
        diff_err.to_string().contains("empty"),
        "sync guard should name the empty Yrs state: {diff_err}"
    );
}

#[test]
fn deferred_structure_change_completes_hydration_before_mutating_and_exporting() {
    let bytes = basic_import_fixture_xlsx();
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");

    let sheet_id = SheetId::from_uuid_str(
        &engine
            .get_all_sheet_ids()
            .first()
            .cloned()
            .expect("deferred fixture should have a sheet"),
    )
    .expect("deferred fixture sheet id should be valid");
    engine
        .structure_change(
            &sheet_id,
            &StructureChange::InsertRows {
                at: 1,
                count: 1,
                new_row_ids: Vec::new(),
            },
        )
        .expect("structural deferred path should complete hydration and insert a row");

    assert!(
        !engine.storage().sheet_order().is_empty(),
        "structural completion must not commit an empty Yrs document"
    );
    let exported = engine
        .export_to_xlsx_bytes()
        .expect("export should succeed after structural deferred completion");
    let parsed = xlsx_api::parse(&exported).expect("structural export should parse");
    assert_eq!(parsed.output.sheets.len(), 1);
}
