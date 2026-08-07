//! Versioned full-state compaction for persisted Yrs documents.
//!
//! Schema v20 removes the persisted `sheets/*/gridIndex/idToPos` inverse.
//! A v19 document is rebuilt into a new Yrs lineage so the discarded map's
//! structs and tombstones cannot remain in the encoded baseline.

use compute_document::schema::{
    CURRENT_SCHEMA_VERSION, KEY_GRID_ID_TO_POS, KEY_GRID_INDEX, KEY_SCHEMA_VERSION, KEY_SECURITY,
    KEY_SHEETS, KEY_WORKBOOK, read_schema_version,
};
use value_types::ComputeError;
use yrs::{Any, Array, ArrayPrelim, ArrayRef, Map, MapPrelim, MapRef, Out, ReadTxn, Transact};

use super::{YrsStorage, new_runtime_doc};

#[derive(Clone)]
enum VisibleValue {
    Any(Any),
    Map(Vec<(String, VisibleValue)>),
    Array(Vec<VisibleValue>),
}

/// Inspect the authoritative schema stamp in raw full-state bytes.
///
/// Missing `workbook.schemaVersion` is reported as schema `0`, matching
/// [`read_schema_version`]. Malformed Yrs updates return an explicit
/// [`ComputeError::Eval`] and are never treated as an unstamped document.
pub fn inspect_yrs_state_schema_version(state: &[u8]) -> Result<u32, ComputeError> {
    let storage = decode_storage(state)?;
    let txn = storage.doc().transact();
    Ok(read_schema_version(&txn, storage.workbook_map()))
}

pub(crate) fn compact_to_current_schema_if_needed(
    source: YrsStorage,
) -> Result<YrsStorage, ComputeError> {
    let source_version = {
        let txn = source.doc().transact();
        read_schema_version(&txn, source.workbook_map())
    };
    let source_is_blank = source_version == 0 && !has_visible_document_state(&source);
    if source_version == CURRENT_SCHEMA_VERSION || source_is_blank {
        return Ok(source);
    }

    let (workbook, sheets, security) = {
        let txn = source.doc().transact();
        let workbook = read_map(source.workbook_map(), &txn, &[KEY_WORKBOOK])?;
        let sheets = read_map(source.sheets(), &txn, &[KEY_SHEETS])?;
        let security = match txn.get_map(KEY_SECURITY) {
            Some(map) => read_map(&map, &txn, &[KEY_SECURITY])?,
            None => Vec::new(),
        };
        (workbook, sheets, security)
    };

    let doc = new_runtime_doc();
    let workbook_map = doc.get_or_insert_map(KEY_WORKBOOK);
    let sheets_map = doc.get_or_insert_map(KEY_SHEETS);
    let security_map = doc.get_or_insert_map(KEY_SECURITY);
    {
        let mut txn = doc.transact_mut();
        write_map_entries(&workbook_map, &mut txn, &workbook);
        workbook_map.insert(
            &mut txn,
            KEY_SCHEMA_VERSION,
            Any::BigInt(CURRENT_SCHEMA_VERSION as i64),
        );
        write_map_entries(&sheets_map, &mut txn, &sheets);
        write_map_entries(&security_map, &mut txn, &security);
    }

    Ok(YrsStorage {
        doc,
        workbook: workbook_map,
        sheets: sheets_map,
    })
}

fn has_visible_document_state(source: &YrsStorage) -> bool {
    let txn = source.doc().transact();
    source.workbook_map().iter(&txn).next().is_some()
        || source.sheets().iter(&txn).next().is_some()
        || txn
            .get_map(KEY_SECURITY)
            .is_some_and(|security| security.iter(&txn).next().is_some())
}

fn decode_storage(state: &[u8]) -> Result<YrsStorage, ComputeError> {
    YrsStorage::from_yrs_state(state).map_err(|error| ComputeError::Eval {
        message: format!("inspect_yrs_state_schema_version: {error}"),
    })
}

fn read_map<T: ReadTxn>(
    map: &MapRef,
    txn: &T,
    path: &[&str],
) -> Result<Vec<(String, VisibleValue)>, ComputeError> {
    let mut entries = Vec::new();
    for (key, out) in map.iter(txn) {
        if should_omit(path, key.as_ref()) {
            continue;
        }
        let mut child_path = path.to_vec();
        child_path.push(key.as_ref());
        entries.push((key.to_string(), read_value(out, txn, &child_path)?));
    }
    Ok(entries)
}

fn read_value<T: ReadTxn>(out: Out, txn: &T, path: &[&str]) -> Result<VisibleValue, ComputeError> {
    match out {
        Out::Any(value) => Ok(VisibleValue::Any(value)),
        Out::YMap(map) => Ok(VisibleValue::Map(read_map(&map, txn, path)?)),
        Out::YArray(array) => {
            let mut values = Vec::with_capacity(array.len(txn) as usize);
            for (index, value) in array.iter(txn).enumerate() {
                let index = index.to_string();
                let mut child_path = path.to_vec();
                child_path.push(index.as_str());
                values.push(read_value(value, txn, &child_path)?);
            }
            Ok(VisibleValue::Array(values))
        }
        unsupported => Err(ComputeError::Eval {
            message: format!(
                "schema v19 to v20 compaction cannot copy unsupported shared type at {}: {unsupported:?}",
                path.join("/")
            ),
        }),
    }
}

fn should_omit(path: &[&str], key: &str) -> bool {
    path.len() == 3
        && path[0] == KEY_SHEETS
        && path[2] == KEY_GRID_INDEX
        && key == KEY_GRID_ID_TO_POS
}

fn write_map_entries(
    map: &MapRef,
    txn: &mut yrs::TransactionMut<'_>,
    entries: &[(String, VisibleValue)],
) {
    for (key, value) in entries {
        write_value_to_map(map, txn, key, value);
    }
}

fn write_value_to_map(
    map: &MapRef,
    txn: &mut yrs::TransactionMut<'_>,
    key: &str,
    value: &VisibleValue,
) {
    match value {
        VisibleValue::Any(value) => {
            map.insert(txn, key, value.clone());
        }
        VisibleValue::Map(entries) => {
            let child: MapRef = map.insert(txn, key, MapPrelim::default());
            write_map_entries(&child, txn, entries);
        }
        VisibleValue::Array(values) => {
            let child: ArrayRef = map.insert(txn, key, ArrayPrelim::default());
            for value in values {
                push_value_to_array(&child, txn, value);
            }
        }
    }
}

fn push_value_to_array(array: &ArrayRef, txn: &mut yrs::TransactionMut<'_>, value: &VisibleValue) {
    match value {
        VisibleValue::Any(value) => {
            array.push_back(txn, value.clone());
        }
        VisibleValue::Map(entries) => {
            let child: MapRef = array.push_back(txn, MapPrelim::default());
            write_map_entries(&child, txn, entries);
        }
        VisibleValue::Array(values) => {
            let child: ArrayRef = array.push_back(txn, ArrayPrelim::default());
            for value in values {
                push_value_to_array(&child, txn, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compute_document::schema::{KEY_GRID_POS_TO_ID, guard_schema_version_with_max};
    use std::sync::Arc;
    use yrs::GetString;

    fn v19_fixture() -> YrsStorage {
        let storage = YrsStorage::new();
        let sheet_id = "00000000000000000000000000000001";
        let cell_id = "00000000000000000000000000000002";
        let mut txn = storage.doc().transact_mut();
        storage
            .workbook_map()
            .insert(&mut txn, KEY_SCHEMA_VERSION, Any::BigInt(19));
        let settings: MapRef =
            storage
                .workbook_map()
                .insert(&mut txn, "workbookSettings", MapPrelim::default());
        settings.insert(&mut txn, "themeId", Any::String(Arc::from("dark")));

        let sheet: MapRef = storage
            .sheets()
            .insert(&mut txn, sheet_id, MapPrelim::default());
        let cells: MapRef = sheet.insert(&mut txn, "cells", MapPrelim::default());
        let cell: MapRef = cells.insert(&mut txn, cell_id, MapPrelim::default());
        cell.insert(&mut txn, "v", Any::Number(42.0));
        cell.insert(&mut txn, "f", Any::String(Arc::from("=SUM(A1:A2)")));
        let properties: MapRef = sheet.insert(&mut txn, "cellProperties", MapPrelim::default());
        let format: MapRef = properties.insert(&mut txn, cell_id, MapPrelim::default());
        format.insert(&mut txn, "bold", Any::Bool(true));
        let comments: MapRef = sheet.insert(&mut txn, "comments", MapPrelim::default());
        let comment: MapRef = comments.insert(&mut txn, "comment-1", MapPrelim::default());
        comment.insert(&mut txn, "cellRef", Any::String(Arc::from(cell_id)));
        comment.insert(&mut txn, "text", Any::String(Arc::from("kept")));

        let grid: MapRef = sheet.insert(&mut txn, KEY_GRID_INDEX, MapPrelim::default());
        let pos_to_id: MapRef = grid.insert(&mut txn, KEY_GRID_POS_TO_ID, MapPrelim::default());
        pos_to_id.insert(&mut txn, "row:col", Any::String(Arc::from(cell_id)));
        let id_to_pos: MapRef = grid.insert(&mut txn, KEY_GRID_ID_TO_POS, MapPrelim::default());
        id_to_pos.insert(&mut txn, cell_id, Any::String(Arc::from("row:col")));

        let security = txn.get_map(KEY_SECURITY).expect("security root");
        let policies: MapRef = security.insert(&mut txn, "policies", MapPrelim::default());
        policies.insert(&mut txn, "policy-1", Any::String(Arc::from("preserved")));
        drop(txn);
        storage
    }

    #[test]
    fn v19_compaction_rebuilds_visible_state_without_inverse_map() {
        let compacted = compact_to_current_schema_if_needed(v19_fixture()).unwrap();
        let txn = compacted.doc().transact();
        assert_eq!(read_schema_version(&txn, compacted.workbook_map()), 20);

        let settings = compacted
            .workbook_map()
            .get(&txn, "workbookSettings")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("workbook settings");
        assert_eq!(
            settings.get(&txn, "themeId"),
            Some(Out::Any(Any::String(Arc::from("dark"))))
        );

        let sheet = compacted
            .sheets()
            .get(&txn, "00000000000000000000000000000001")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("sheet");
        let grid = sheet
            .get(&txn, KEY_GRID_INDEX)
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("grid index");
        assert!(matches!(
            grid.get(&txn, KEY_GRID_POS_TO_ID),
            Some(Out::YMap(_))
        ));
        assert!(grid.get(&txn, KEY_GRID_ID_TO_POS).is_none());

        let cells = sheet
            .get(&txn, "cells")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("cells");
        let cell = cells
            .get(&txn, "00000000000000000000000000000002")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("cell identity preserved");
        assert_eq!(cell.get(&txn, "v"), Some(Out::Any(Any::Number(42.0))));
        assert_eq!(
            cell.get(&txn, "f"),
            Some(Out::Any(Any::String(Arc::from("=SUM(A1:A2)"))))
        );

        let properties = sheet
            .get(&txn, "cellProperties")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("cell properties");
        assert!(
            properties
                .get(&txn, "00000000000000000000000000000002")
                .is_some()
        );
        let comments = sheet
            .get(&txn, "comments")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("comments");
        assert!(comments.get(&txn, "comment-1").is_some());

        let security = txn.get_map(KEY_SECURITY).expect("security");
        let policies = security
            .get(&txn, "policies")
            .and_then(|out| out.cast::<MapRef>().ok())
            .expect("policies");
        assert_eq!(
            policies.get(&txn, "policy-1"),
            Some(Out::Any(Any::String(Arc::from("preserved"))))
        );
    }

    #[test]
    fn older_stamped_documents_are_also_compacted_before_becoming_v20() {
        let storage = v19_fixture();
        {
            let mut txn = storage.doc().transact_mut();
            storage
                .workbook_map()
                .insert(&mut txn, KEY_SCHEMA_VERSION, Any::BigInt(16));
        }

        let compacted = compact_to_current_schema_if_needed(storage).unwrap();
        let txn = compacted.doc().transact();
        assert_eq!(read_schema_version(&txn, compacted.workbook_map()), 20);
        let sheet = compacted
            .sheets()
            .get(&txn, "00000000000000000000000000000001")
            .and_then(|out| out.cast::<MapRef>().ok())
            .unwrap();
        let grid = sheet
            .get(&txn, KEY_GRID_INDEX)
            .and_then(|out| out.cast::<MapRef>().ok())
            .unwrap();
        assert!(grid.get(&txn, KEY_GRID_ID_TO_POS).is_none());
    }

    #[test]
    fn blank_unstamped_provider_target_stays_uncommitted() {
        let storage = YrsStorage::new();
        let storage = compact_to_current_schema_if_needed(storage).unwrap();
        let txn = storage.doc().transact();
        assert_eq!(read_schema_version(&txn, storage.workbook_map()), 0);
    }

    #[test]
    fn compacted_full_state_is_byte_stable_on_second_load() {
        let compacted = compact_to_current_schema_if_needed(v19_fixture()).unwrap();
        let first = compute_collab::encode_full_state(compacted.doc());
        let second_storage = YrsStorage::from_yrs_state(&first).unwrap();
        let second = compute_collab::encode_full_state(second_storage.doc());
        assert_eq!(first, second);
    }

    #[test]
    fn thousand_cell_fixture_reports_full_state_reduction() {
        const CELL_COUNT: u32 = 1_000;
        let storage = YrsStorage::new();
        let mut txn = storage.doc().transact_mut();
        storage
            .workbook_map()
            .insert(&mut txn, KEY_SCHEMA_VERSION, Any::BigInt(19));
        let sheet: MapRef = storage.sheets().insert(
            &mut txn,
            "00000000000000000000000000000001",
            MapPrelim::default(),
        );
        let cells: MapRef = sheet.insert(&mut txn, "cells", MapPrelim::default());
        let grid: MapRef = sheet.insert(&mut txn, KEY_GRID_INDEX, MapPrelim::default());
        let pos_to_id: MapRef = grid.insert(&mut txn, KEY_GRID_POS_TO_ID, MapPrelim::default());
        let id_to_pos: MapRef = grid.insert(&mut txn, KEY_GRID_ID_TO_POS, MapPrelim::default());

        for row in 0..CELL_COUNT {
            let cell_id = format!("{:032x}", u128::from(row) + 2);
            let position = format!("{:032x}:{:032x}", u128::from(row) + 10_000, 1_u128);
            let cell: MapRef = cells.insert(&mut txn, cell_id.as_str(), MapPrelim::default());
            cell.insert(&mut txn, "v", Any::Number(f64::from(row)));
            pos_to_id.insert(
                &mut txn,
                position.as_str(),
                Any::String(Arc::from(cell_id.as_str())),
            );
            id_to_pos.insert(
                &mut txn,
                cell_id.as_str(),
                Any::String(Arc::from(position.as_str())),
            );
        }
        drop(txn);

        let before = compute_collab::encode_full_state(storage.doc()).len();
        let compacted = compact_to_current_schema_if_needed(storage).unwrap();
        let after = compute_collab::encode_full_state(compacted.doc()).len();
        let reduction = before - after;
        let reduction_percent = reduction as f64 * 100.0 / before as f64;

        eprintln!(
            "schema-v20-encode-full-state: cells={CELL_COUNT} before={before} after={after} reduction={reduction} percent={reduction_percent:.2}"
        );
        assert!(
            after < before,
            "v20 fixture must omit redundant inverse state"
        );
    }

    #[test]
    fn inspection_reads_original_stamp_without_compacting() {
        let fixture = v19_fixture();
        let bytes = compute_collab::encode_full_state(fixture.doc());
        assert_eq!(inspect_yrs_state_schema_version(&bytes).unwrap(), 19);

        let missing = YrsStorage::new();
        let missing_bytes = compute_collab::encode_full_state(missing.doc());
        assert_eq!(inspect_yrs_state_schema_version(&missing_bytes).unwrap(), 0);
        assert!(inspect_yrs_state_schema_version(b"not yrs").is_err());
    }

    #[test]
    fn unsupported_shared_types_fail_compaction_explicitly() {
        let storage = v19_fixture();
        let sheet = {
            let txn = storage.doc().transact();
            storage
                .sheets()
                .get(&txn, "00000000000000000000000000000001")
                .and_then(|out| out.cast::<MapRef>().ok())
                .unwrap()
        };
        let mut txn = storage.doc().transact_mut();
        let nested_text = sheet.insert(&mut txn, "unsupported", yrs::TextPrelim::new("text"));
        assert_eq!(nested_text.get_string(&txn), "text");
        drop(txn);

        let err = compact_to_current_schema_if_needed(storage)
            .err()
            .expect("shared text must not be silently dropped");
        assert!(err.to_string().contains("unsupported shared type"));
    }

    #[test]
    fn v20_stamp_is_refused_under_simulated_v19_max() {
        let compacted = compact_to_current_schema_if_needed(v19_fixture()).unwrap();
        let txn = compacted.doc().transact();
        assert!(guard_schema_version_with_max(&txn, compacted.workbook_map(), 19).is_err());
    }
}
