use yrs::{Any, Array, Map, Origin, Out, Transact};

use cell_types::SheetId;
use compute_document::hex::id_to_hex;
use compute_document::schema::{
    KEY_CELLS, KEY_COL_ORDER, KEY_COMMENTS, KEY_GRID_INDEX, KEY_GRID_POS_TO_ID, KEY_ROW_ORDER,
};
use compute_document::undo::ORIGIN_USER_EDIT;
use domain_types::domain::comment::{Comment, RichTextRun};
use domain_types::yrs_schema::comment as comment_schema;

use crate::storage::YrsStorage;

pub(super) fn make_sheet_id(n: u128) -> SheetId {
    SheetId::from_raw(n)
}

pub(super) fn storage_with_sheet() -> (YrsStorage, SheetId) {
    let mut storage = YrsStorage::new();
    let mut mirror = crate::mirror::CellMirror::new();
    let sheet_id = make_sheet_id(1);
    storage
        .add_sheet(&mut mirror, sheet_id, "Sheet1", 100, 26)
        .expect("add_sheet should succeed");
    (storage, sheet_id)
}

pub(super) fn simple_runs(text: &str) -> Vec<RichTextRun> {
    vec![RichTextRun {
        text: text.to_string(),
        ..Default::default()
    }]
}

pub(super) fn add_cell_to_sheet(storage: &YrsStorage, sheet_id: &SheetId, cell_id_key: &str) {
    let sheet_hex = id_to_hex(sheet_id.as_u128());
    let mut txn = storage
        .doc()
        .transact_mut_with(Origin::from(ORIGIN_USER_EDIT));
    let sheet_map = match storage.sheets_ref().get(&txn, &sheet_hex) {
        Some(Out::YMap(m)) => m,
        _ => panic!("sheet not found"),
    };
    let cells_map = match sheet_map.get(&txn, KEY_CELLS) {
        Some(Out::YMap(m)) => m,
        _ => panic!("cells map not found"),
    };
    let cell_prelim = yrs::MapPrelim::from([("v", Any::Number(0.0))]);
    cells_map.insert(&mut txn, cell_id_key, cell_prelim);
}

pub(super) fn add_grid_index_cell(storage: &YrsStorage, sheet_id: &SheetId, cell_id_key: &str) {
    let sheet_hex = id_to_hex(sheet_id.as_u128());
    let mut txn = storage
        .doc()
        .transact_mut_with(Origin::from(ORIGIN_USER_EDIT));
    let sheet_map = match storage.sheets_ref().get(&txn, &sheet_hex) {
        Some(Out::YMap(m)) => m,
        _ => panic!("sheet not found"),
    };
    let grid_index = match sheet_map.get(&txn, KEY_GRID_INDEX) {
        Some(Out::YMap(m)) => m,
        _ => panic!("grid index not found"),
    };
    let pos_to_id = match grid_index.get(&txn, KEY_GRID_POS_TO_ID) {
        Some(Out::YMap(m)) => m,
        _ => panic!("pos_to_id map not found"),
    };
    let row_order = match sheet_map.get(&txn, KEY_ROW_ORDER) {
        Some(Out::YArray(a)) => a,
        _ => panic!("row order not found"),
    };
    let col_order = match sheet_map.get(&txn, KEY_COL_ORDER) {
        Some(Out::YArray(a)) => a,
        _ => panic!("column order not found"),
    };
    let Some(Out::Any(Any::String(row_hex))) = row_order.get(&txn, 0) else {
        panic!("row identity not found");
    };
    let Some(Out::Any(Any::String(col_hex))) = col_order.get(&txn, 0) else {
        panic!("column identity not found");
    };
    let pos_key = format!("{row_hex}:{col_hex}");
    pos_to_id.insert(
        &mut txn,
        pos_key,
        Any::String(std::sync::Arc::from(cell_id_key)),
    );
}

pub(super) fn insert_comment_with_key(
    storage: &YrsStorage,
    sheet_id: &SheetId,
    key: &str,
    comment: &Comment,
) {
    let sheet_hex = id_to_hex(sheet_id.as_u128());
    let mut txn = storage
        .doc()
        .transact_mut_with(Origin::from(ORIGIN_USER_EDIT));
    let sheet_map = match storage.sheets_ref().get(&txn, &sheet_hex) {
        Some(Out::YMap(m)) => m,
        _ => panic!("sheet not found"),
    };
    let comments_map = match sheet_map.get(&txn, KEY_COMMENTS) {
        Some(Out::YMap(m)) => m,
        _ => panic!("comments map not found"),
    };
    let prelim: yrs::MapPrelim = comment_schema::to_yrs_prelim(comment).into_iter().collect();
    comments_map.insert(&mut txn, key, prelim);
}
