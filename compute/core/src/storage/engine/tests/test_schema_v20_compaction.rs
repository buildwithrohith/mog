use super::helpers::{cell_value_at, num, sheet_id, simple_snapshot};
use crate::storage::engine::YrsComputeEngine;
use compute_document::schema::{
    KEY_GRID_ID_TO_POS, KEY_GRID_INDEX, KEY_GRID_POS_TO_ID, KEY_SCHEMA_VERSION,
};
use std::sync::Arc;
use yrs::{Any, Map, MapPrelim, MapRef, Out, Transact};

#[test]
fn v19_full_state_load_preserves_values_formulas_and_cell_ids_in_fresh_v20_doc() {
    let (source, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    let sid = sheet_id();
    let expected_ids: Vec<String> = [(0, 0), (0, 1), (1, 0)]
        .into_iter()
        .map(|(row, col)| source.get_cell_id_at(&sid, row, col).unwrap())
        .collect();

    // Model persisted v19 bytes by restoring the old schema stamp and inverse
    // map on an otherwise representative engine document.
    {
        let mut txn = source.storage().doc().transact_mut();
        source
            .storage()
            .workbook_map()
            .insert(&mut txn, KEY_SCHEMA_VERSION, Any::BigInt(19));
    }
    let inverse_plans: Vec<(MapRef, Vec<(String, String)>)> = {
        let txn = source.storage().doc().transact();
        source
            .storage()
            .sheets()
            .iter(&txn)
            .filter_map(|(_sheet_hex, sheet_out)| {
                let Out::YMap(sheet) = sheet_out else {
                    return None;
                };
                let Some(Out::YMap(grid)) = sheet.get(&txn, KEY_GRID_INDEX) else {
                    return None;
                };
                let Some(Out::YMap(pos_to_id)) = grid.get(&txn, KEY_GRID_POS_TO_ID) else {
                    return None;
                };
                let inverse_entries = pos_to_id
                    .iter(&txn)
                    .filter_map(|(position, value)| match value {
                        Out::Any(Any::String(cell_id)) => {
                            Some((cell_id.to_string(), position.to_string()))
                        }
                        _ => None,
                    })
                    .collect();
                Some((grid, inverse_entries))
            })
            .collect()
    };
    {
        let mut txn = source.storage().doc().transact_mut();
        for (grid, inverse_entries) in inverse_plans {
            let inverse: MapRef = grid.insert(&mut txn, KEY_GRID_ID_TO_POS, MapPrelim::default());
            for (cell_id, position) in inverse_entries {
                inverse.insert(&mut txn, cell_id, Any::String(Arc::from(position.as_str())));
            }
        }
    }

    let v19_bytes = compute_collab::encode_full_state(source.storage().doc());
    let (reloaded, _) = YrsComputeEngine::from_yrs_state(&v19_bytes).unwrap();

    assert_eq!(cell_value_at(&reloaded, &sid, 0, 0), num(10.0));
    assert_eq!(cell_value_at(&reloaded, &sid, 0, 1), num(20.0));
    assert_eq!(cell_value_at(&reloaded, &sid, 1, 0), num(30.0));
    assert_eq!(
        reloaded.get_cell_info(&sid, 1, 0).unwrap().formula,
        Some("=A1+B1".to_string())
    );
    let actual_ids: Vec<String> = [(0, 0), (0, 1), (1, 0)]
        .into_iter()
        .map(|(row, col)| reloaded.get_cell_id_at(&sid, row, col).unwrap())
        .collect();
    assert_eq!(actual_ids, expected_ids);

    let txn = reloaded.storage().doc().transact();
    for (_sheet_hex, sheet_out) in reloaded.storage().sheets().iter(&txn) {
        let Out::YMap(sheet) = sheet_out else {
            continue;
        };
        let Some(Out::YMap(grid)) = sheet.get(&txn, KEY_GRID_INDEX) else {
            continue;
        };
        assert!(
            grid.get(&txn, KEY_GRID_ID_TO_POS).is_none(),
            "compacted v20 sheet must not retain the v19 inverse map"
        );
    }
    drop(txn);

    let first_v20_bytes = compute_collab::encode_full_state(reloaded.storage().doc());
    let (second_load, _) = YrsComputeEngine::from_yrs_state(&first_v20_bytes).unwrap();
    let second_v20_bytes = compute_collab::encode_full_state(second_load.storage().doc());
    assert_eq!(first_v20_bytes, second_v20_bytes);
}
