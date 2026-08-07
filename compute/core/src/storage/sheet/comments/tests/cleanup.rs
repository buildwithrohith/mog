use super::super::*;
use super::helpers::*;

fn empty_grid(sheet_id: cell_types::SheetId) -> compute_document::identity::GridIndex {
    compute_document::identity::GridIndex::new(
        sheet_id,
        100,
        26,
        std::sync::Arc::new(cell_types::IdAllocator::new()),
    )
}

#[test]
fn test_validate_and_clean_orphaned_comments() {
    let (storage, sheet_id) = storage_with_sheet();
    let doc = storage.doc();
    let sheets = &storage.sheets_ref();
    let grid = empty_grid(sheet_id);
    add_cell_to_sheet(&storage, &sheet_id, "existing-cell");
    add_comment(
        doc,
        sheets,
        &sheet_id,
        "existing-cell",
        simple_runs("Valid comment"),
        "Alice",
        AddCommentOptions::default(),
        &crate::storage::STORAGE_ID_ALLOC,
    )
    .unwrap();
    add_comment(
        doc,
        sheets,
        &sheet_id,
        "orphaned-cell",
        simple_runs("Orphaned comment"),
        "Bob",
        AddCommentOptions::default(),
        &crate::storage::STORAGE_ID_ALLOC,
    )
    .unwrap();
    assert_eq!(get_comment_count(doc, sheets, &sheet_id), 2);
    let removed = validate_and_clean_comments(doc, sheets, &sheet_id, &grid);
    assert_eq!(removed, 1);
    assert_eq!(get_comment_count(doc, sheets, &sheet_id), 1);
    assert!(has_comments(doc, sheets, &sheet_id, "existing-cell"));
    assert!(!has_comments(doc, sheets, &sheet_id, "orphaned-cell"));
}

#[test]
fn test_validate_and_clean_no_orphans() {
    let (storage, sheet_id) = storage_with_sheet();
    let doc = storage.doc();
    let sheets = &storage.sheets_ref();
    let grid = empty_grid(sheet_id);
    add_cell_to_sheet(&storage, &sheet_id, "cell-001");
    add_comment(
        doc,
        sheets,
        &sheet_id,
        "cell-001",
        simple_runs("Valid"),
        "Alice",
        AddCommentOptions::default(),
        &crate::storage::STORAGE_ID_ALLOC,
    )
    .unwrap();
    let removed = validate_and_clean_comments(doc, sheets, &sheet_id, &grid);
    assert_eq!(removed, 0);
    assert_eq!(get_comment_count(doc, sheets, &sheet_id), 1);
}

#[test]
fn test_validate_and_clean_preserves_cell_from_grid_index() {
    let (storage, sheet_id) = storage_with_sheet();
    let doc = storage.doc();
    let sheets = &storage.sheets_ref();
    let cell_ref = "00000000000000000000000000000123";
    add_grid_index_cell(&storage, &sheet_id, cell_ref);
    let mut grid = empty_grid(sheet_id);
    let cell_id = cell_types::CellId::from_raw(
        compute_document::hex::hex_to_id(cell_ref).expect("valid cell identity"),
    );
    grid.register_cell(cell_id, 0, 0);
    add_comment(
        doc,
        sheets,
        &sheet_id,
        cell_ref,
        simple_runs("Valid through grid index"),
        "Alice",
        AddCommentOptions::default(),
        &crate::storage::STORAGE_ID_ALLOC,
    )
    .unwrap();
    add_comment(
        doc,
        sheets,
        &sheet_id,
        "orphaned-cell",
        simple_runs("Orphan"),
        "Bob",
        AddCommentOptions::default(),
        &crate::storage::STORAGE_ID_ALLOC,
    )
    .unwrap();

    let removed = validate_and_clean_comments(doc, sheets, &sheet_id, &grid);
    assert_eq!(removed, 1);
    assert!(has_comments(doc, sheets, &sheet_id, cell_ref));
    assert!(!has_comments(doc, sheets, &sheet_id, "orphaned-cell"));
}
