/* tslint:disable */
/* eslint-disable */

export function chart_apply_transforms(data: any, transforms: any): any;

export function chart_compute_bins(values: any, maxbins: any, step: any, nice: any): any;

export function chart_compute_density(values: any, bandwidth: any, steps: any): any;

export function chart_compute_regression(points: any, method: any, degree: any, options: any): any;

export function chart_compute_stacking(inputs: any, mode: any): any;

export function chart_compute_statistics(values: any): any;

export function compute_active_principal(doc_id: string): any;

export function compute_add_calculated_column(doc_id: string, table_name: string, column_name: string, formula: string): any;

export function compute_add_cf_rule(doc_id: string, sheet_id: any, rule: any): any;

export function compute_add_comment(doc_id: string, sheet_id: any, cell_id: string, text: string, author: string, author_id: any, parent_id: any, comment_type: any): any;

export function compute_add_comment_by_position(doc_id: string, sheet_id: any, row: number, col: number, text: string, author: string, author_id: any, parent_id: any, comment_type: any): any;

export function compute_add_compute_sheet(doc_id: string, snapshot: any): any;

export function compute_add_horizontal_page_break(doc_id: string, sheet_id: any, row: number): any;

export function compute_add_rule_to_cf(doc_id: string, sheet_id: any, format_id: string, rule: any): any;

export function compute_add_slicer_style(doc_id: string, name: string, style: any, make_unique_name: boolean): any;

export function compute_add_sparkline(doc_id: string, sheet_id: any, sparkline: any): any;

export function compute_add_sparkline_group(doc_id: string, sheet_id: any, group: any): any;

export function compute_add_table_column(doc_id: string, table_name: string, column_name: string, position: number): any;

export function compute_add_table_data_row(doc_id: string, table_name: string, relative_row: any): any;

export function compute_add_vertical_page_break(doc_id: string, sheet_id: any, col: number): any;

export function compute_apply_advanced_filter(doc_id: string, sheet_id: any, request: any): any;

export function compute_apply_auto_expansion(doc_id: string, sheet_id: any, table_name: string): any;

export function compute_apply_calculated_formulas_to_row(doc_id: string, table_name: string, row: number, formulas: any): any;

export function compute_apply_changes(doc_id: string, changes: any, skip_cycle_check: boolean): any;

export function compute_apply_filter(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_apply_scenario(doc_id: string, scenario_id: string): any;

export function compute_apply_sync_update(doc_id: string, update: Uint8Array, sync_context: any): any;

export function compute_auto_fill(doc_id: string, sheet_id: any, request: any): any;

export function compute_auto_fill_preview(doc_id: string, sheet_id: any, request: any): any;

export function compute_auto_fit_column_and_set(doc_id: string, sheet_id: any, col: number): any;

export function compute_auto_fit_columns_and_set(doc_id: string, sheet_id: any, cols: any): any;

export function compute_auto_fit_rows_and_set(doc_id: string, sheet_id: any, rows: any): any;

export function compute_auto_outline(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_batch_clear_cells(doc_id: string, cell_ids: any): any;

export function compute_batch_set_cells(doc_id: string, edits: any, skip_cycle_check: boolean): any;

export function compute_batch_set_cells_by_position(doc_id: string, edits: any, skip_cycle_check: boolean): any;

export function compute_begin_undo_group(doc_id: string): any;

export function compute_bring_chart_forward(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_bring_chart_to_front(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_bring_floating_object_forward(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_bring_floating_object_to_front(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_can_do_structure_op(doc_id: string, sheet_id: any, operation: string): boolean;

export function compute_can_edit_cell(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_can_redo(doc_id: string): boolean;

export function compute_can_undo(doc_id: string): boolean;

export function compute_capture_screenshot(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, dpr: number, show_headers: boolean, show_gridlines: boolean, max_width: any, max_height: any): Uint8Array;

export function compute_cf_intersect_ranges(doc_id: string, a: any, b: any): any;

export function compute_cf_is_valid_range(doc_id: string, range: any): boolean;

export function compute_cf_range_contains(doc_id: string, outer: any, inner: any): boolean;

export function compute_cf_ranges_overlap(doc_id: string, a: any, b: any): boolean;

export function compute_cf_subtract_range(doc_id: string, original: any, subtract: any): any;

export function compute_check_merge_data_loss(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_check_sort_range_merges(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_classify_value_type(value: any): string;

export function compute_clear_all_column_filters(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_clear_all_comments(doc_id: string, sheet_id: any): any;

export function compute_clear_all_filters(doc_id: string, sheet_id: any): any;

export function compute_clear_all_grouping(doc_id: string, sheet_id: any): any;

export function compute_clear_all_merges(doc_id: string, sheet_id: any): any;

export function compute_clear_all_page_breaks(doc_id: string, sheet_id: any): any;

export function compute_clear_cell_format(doc_id: string, sheet_id: any, cell_id: any): any;

export function compute_clear_cf_formats_for_sheet(doc_id: string, sheet_id: any): any;

export function compute_clear_col_format(doc_id: string, sheet_id: any, col: number): any;

export function compute_clear_column_filter(doc_id: string, sheet_id: any, filter_id: string, header_col: number): any;

export function compute_clear_column_grouping(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_clear_column_schema(doc_id: string, sheet_id: any, col_index: number): any;

export function compute_clear_format_for_ranges(doc_id: string, sheet_id: any, ranges: any): any;

export function compute_clear_hyperlinks_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_clear_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_clear_range_and_return_ids(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_clear_range_by_position(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_clear_range_with_mode(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, mode: string): any;

export function compute_clear_row_grouping(doc_id: string, sheet_id: any, start_row: number, end_row: number): any;

export function compute_clear_schemas(doc_id: string): any;

export function compute_clear_slicer_selection(doc_id: string, sheet_id: any, slicer_id: string): any;

export function compute_clear_sparklines_for_sheet(doc_id: string, sheet_id: any): any;

export function compute_clear_sparklines_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_collapse_all_groups(doc_id: string, sheet_id: any): any;

export function compute_complete_deferred_hydration(doc_id: string): any;

export function compute_compute_all_object_bounds(doc_id: string, sheet_id: any): any;

export function compute_compute_dynamic_filter_serial_range(doc_id: string, rule: any): any;

export function compute_convert_note_to_thread(doc_id: string, sheet_id: any, comment_id: string): any;

export function compute_convert_table_to_range(doc_id: string, table_name: string): any;

export function compute_copy_range(doc_id: string, source_sheet_id: any, src_start_row: number, src_start_col: number, src_end_row: number, src_end_col: number, target_sheet_id: any, target_row: number, target_col: number, copy_type: any, skip_blanks: boolean, transpose: boolean): any;

export function compute_copy_sheet(doc_id: string, sheet_id: any, new_name: string): any;

export function compute_count_visible_sheets(doc_id: string): number;

export function compute_create_binding(doc_id: string, sheet_id: any, binding: any): any;

export function compute_create_chart(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_custom_cell_style(doc_id: string, style: any): any;

export function compute_create_custom_table_style(doc_id: string, style: any): any;

export function compute_create_data_table(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, input: any): any;

export function compute_create_default_sheet(doc_id: string, name: string): any;

export function compute_create_default_sheet_with_default_col_width(doc_id: string, name: string, default_col_width_px: number): any;

export function compute_create_filter(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_floating_object(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_floating_object_group(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_named_range(doc_id: string, input: any): any;

export function compute_create_scenario(doc_id: string, input: any): any;

export function compute_create_shape(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_sheet(doc_id: string, name: string): any;

export function compute_create_sheet_with_default_col_width(doc_id: string, name: string, default_col_width_px: number): any;

export function compute_create_slicer(doc_id: string, sheet_id: any, config: any): any;

export function compute_create_subtotals(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, options: any): any;

export function compute_create_table(doc_id: string, sheet_id: any, name: string, start_row: number, start_col: number, end_row: number, end_col: number, columns: any, has_headers: boolean): any;

export function compute_create_table_lifecycle(doc_id: string, sheet_id: any, requested_name: any, start_row: number, start_col: number, end_row: number, end_col: number, columns: any, has_headers: boolean, style: any): any;

export function compute_current_state_vector(doc_id: string): Uint8Array;

export function compute_data_table(doc_id: string, params: any): any;

export function compute_delete_cells_with_shift(doc_id: string, sheet_id: any, row: number, col: number, row_count: number, col_count: number, shift_left: boolean): any;

export function compute_delete_cf_rule(doc_id: string, sheet_id: any, rule_id: string): any;

export function compute_delete_chart(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_delete_comment(doc_id: string, sheet_id: any, comment_id: string): any;

export function compute_delete_comments_for_cell(doc_id: string, sheet_id: any, cell_id: string): any;

export function compute_delete_comments_for_cell_by_position(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_delete_custom_cell_style(doc_id: string, id: string): any;

export function compute_delete_custom_table_style(doc_id: string, style_name: string): any;

export function compute_delete_filter(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_delete_floating_object(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_delete_floating_object_group(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_delete_range_schema(doc_id: string, sheet_id: any, schema_id: string): any;

export function compute_delete_rule_from_cf(doc_id: string, sheet_id: any, format_id: string, rule_id: string): any;

export function compute_delete_sheet(doc_id: string, sheet_id: any): any;

export function compute_delete_slicer(doc_id: string, sheet_id: any, slicer_id: string): any;

export function compute_delete_slicer_style(doc_id: string, name: string): any;

export function compute_delete_slicers(doc_id: string, sheet_id: any, slicer_ids: any): any;

export function compute_delete_sparkline(doc_id: string, sheet_id: any, sparkline_id: string): any;

export function compute_delete_sparkline_group(doc_id: string, sheet_id: any, group_id: string, delete_sparklines: boolean): any;

export function compute_delete_table(doc_id: string, table_name: string): any;

export function compute_destroy(id: string): void;

export function compute_detect_auto_expansion(doc_id: string, sheet_id: any, table_name: string): any;

export function compute_detect_format_type(format_code: string): string;

export function compute_drain_pending_updates(doc_id: string): any;

export function compute_duplicate_floating_object_typed(doc_id: string, sheet_id: any, object_id: string, offset_x: number, offset_y: number): any;

export function compute_duplicate_slicer_style(doc_id: string, name: string): any;

export function compute_encode_diff(doc_id: string, remote_sv: Uint8Array): Uint8Array;

export function compute_encode_state_vector(doc_id: string): Uint8Array;

export function compute_end_undo_group(doc_id: string): any;

export function compute_eval_cf(doc_id: string, sheet_id: any, rules: any): any;

export function compute_evaluate_expression(doc_id: string, sheet_id: any, expression: string): any;

export function compute_expand_all_groups(doc_id: string, sheet_id: any): any;

export function compute_export_to_xlsx_bytes(doc_id: string): Uint8Array;

export function compute_export_to_xlsx_bytes_context_stripped(doc_id: string): Uint8Array;

export function compute_find_all_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, options: any): any;

export function compute_find_cells_by_formula(doc_id: string, sheet_id: any, pattern: string): any;

export function compute_find_cells_by_value(doc_id: string, sheet_id: any, value: string, start_row: any, start_col: any, end_row: any, end_col: any): any;

export function compute_find_connectors_for_shape(doc_id: string, sheet_id: any, shape_id: string): any;

export function compute_find_data_edge(doc_id: string, sheet_id: any, row: number, col: number, direction: string): any;

export function compute_find_disconnected_slicers(doc_id: string, slicer_list: any, existing_table_ids: any): any;

export function compute_find_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, options: any): any;

export function compute_find_last_column(doc_id: string, sheet_id: any, row: number): any;

export function compute_find_last_row(doc_id: string, sheet_id: any, col: number): any;

export function compute_find_slicers_for_table(doc_id: string, slicer_list: any, table_id: string): any;

export function compute_flash_fill(doc_id: string, sheet_id: any, request: any): any;

export function compute_flip_floating_object_typed(doc_id: string, sheet_id: any, object_id: string, axis: any): any;

export function compute_flush_undo_capture(doc_id: string): any;

export function compute_format_cell_value_for_display(doc_id: string, sheet_id: any, row: number, col: number): string;

export function compute_format_values(doc_id: string, entries: any): any;

export function compute_freeze_columns(doc_id: string, sheet_id: any, count: number): any;

export function compute_freeze_rows(doc_id: string, sheet_id: any, count: number): any;

export function compute_full_recalc(doc_id: string, options: any): any;

export function compute_get_active_cell(doc_id: string, sheet_id: any, cell_id: any): any;

export function compute_get_active_filter_count(doc_id: string, sheet_id: any): number;

export function compute_get_active_filters(doc_id: string, sheet_id: any): any;

export function compute_get_active_scenario_state(doc_id: string): any;

export function compute_get_affected_columns_by_group(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_affected_rows_by_group(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_all_bindings(doc_id: string, sheet_id: any): any;

export function compute_get_all_cells_yrs(doc_id: string, sheet_id: any): any;

export function compute_get_all_cf_rules(doc_id: string, sheet_id: any): any;

export function compute_get_all_charts(doc_id: string, sheet_id: any): any;

export function compute_get_all_column_schemas(doc_id: string, sheet_id: any): any;

export function compute_get_all_comments(doc_id: string, sheet_id: any): any;

export function compute_get_all_comments_workbook(doc_id: string): any;

export function compute_get_all_custom_cell_styles(doc_id: string): any;

export function compute_get_all_custom_table_styles(doc_id: string): any;

export function compute_get_all_floating_object_groups_typed(doc_id: string, sheet_id: any): any;

export function compute_get_all_floating_objects_typed(doc_id: string, sheet_id: any): any;

export function compute_get_all_in_z_order(doc_id: string, sheet_id: any): any;

export function compute_get_all_merges_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_all_named_ranges_wire(doc_id: string): any;

export function compute_get_all_notes(doc_id: string, sheet_id: any): any;

export function compute_get_all_pivot_tables_workbook(doc_id: string): any;

export function compute_get_all_scenarios(doc_id: string): any;

export function compute_get_all_sheet_ids(doc_id: string): any;

export function compute_get_all_slicers(doc_id: string, sheet_id: any): any;

export function compute_get_all_slicers_workbook(doc_id: string): any;

export function compute_get_all_tables_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_all_tables_workbook(doc_id: string): any;

export function compute_get_binding(doc_id: string, sheet_id: any, binding_id: string): any;

export function compute_get_bindings_for_connection(doc_id: string, connection_id: string): any;

export function compute_get_calc_mode(doc_id: string): string;

export function compute_get_calculation_settings(doc_id: string): any;

export function compute_get_cell_annotation_by_position(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cell_count(doc_id: string, sheet_id: any): number;

export function compute_get_cell_data(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cell_data_by_id_hex(doc_id: string, sheet_id: any, cell_id_hex: string): any;

export function compute_get_cell_format(doc_id: string, sheet_id: any, cell_id: any, row: number, col: number): any;

export function compute_get_cell_format_with_cf(doc_id: string, sheet_id: any, cell_id: any, row: number, col: number): any;

export function compute_get_cell_id_at(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cell_id_at_yrs(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cell_info(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cell_position(doc_id: string, sheet_id: any, cell_id_hex: string): any;

export function compute_get_cell_value(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_cells_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_cells_in_range_yrs(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_cf_preset_by_id(doc_id: string, id: string): any;

export function compute_get_cf_presets(): any;

export function compute_get_cf_rules_for_cell(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_chart(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_get_charts_in_z_order(doc_id: string, sheet_id: any): any;

export function compute_get_charts_linked_to_table(doc_id: string, sheet_id: any, table_id: string): any;

export function compute_get_col_at_pixel(doc_id: string, sheet_id: any, x: number): number;

export function compute_get_col_formats(doc_id: string, sheet_id: any, cols: any): any;

export function compute_get_col_position(doc_id: string, sheet_id: any, col: number): number;

export function compute_get_col_width_chars_query(doc_id: string, sheet_id: any, col: number): number;

export function compute_get_col_width_from_index(doc_id: string, sheet_id: any, col: number): number;

export function compute_get_col_width_query(doc_id: string, sheet_id: any, col: number): number;

export function compute_get_col_widths_batch(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_get_col_widths_batch_chars(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_get_color_scale_presets(): any;

export function compute_get_column_outline_levels(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_get_column_schema(doc_id: string, sheet_id: any, col_index: number): any;

export function compute_get_comment(doc_id: string, sheet_id: any, comment_id: string): any;

export function compute_get_comment_count(doc_id: string, sheet_id: any): number;

export function compute_get_comment_thread(doc_id: string, sheet_id: any, thread_id: string): any;

export function compute_get_comments_for_cell(doc_id: string, sheet_id: any, cell_id: string): any;

export function compute_get_comments_for_cell_by_position(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_conditional_format(doc_id: string, sheet_id: any, format_id: string): any;

export function compute_get_current_region(doc_id: string, sheet_id: any, start_row: number, start_col: number): any;

export function compute_get_custom_setting(doc_id: string, key: string): any;

export function compute_get_data_bar_presets(): any;

export function compute_get_data_bounds(doc_id: string, sheet_id: any): any;

export function compute_get_data_bounds_for_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, is_full_column: boolean, is_full_row: boolean): any;

export function compute_get_default_col_width(doc_id: string, sheet_id: any): number;

export function compute_get_default_col_width_chars(doc_id: string, sheet_id: any): number;

export function compute_get_default_font(doc_id: string): any;

export function compute_get_default_pivot_table_style(doc_id: string): any;

export function compute_get_default_row_height(doc_id: string, sheet_id: any): number;

export function compute_get_default_slicer_style(doc_id: string): any;

export function compute_get_default_table_style_id(doc_id: string): any;

export function compute_get_dependents(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_display_text_2d(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_display_value(doc_id: string, sheet_id: any, row: number, col: number): string;

export function compute_get_displayed_cell_properties(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_displayed_range_properties(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_document_properties(doc_id: string): any;

export function compute_get_effective_value(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_filter(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_get_filter_count(doc_id: string, sheet_id: any): number;

export function compute_get_filter_header_info(doc_id: string, sheet_id: any): any;

export function compute_get_filter_hidden_rows(doc_id: string, sheet_id: any): any;

export function compute_get_filter_sort_state(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_get_filtered_record_count(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_get_filters_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_first_sheet_id(doc_id: string): any;

export function compute_get_floating_object(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_get_floating_object_group(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_floating_object_group_typed(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_floating_object_groups_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_floating_object_max_z_index(doc_id: string, sheet_id: any): number;

export function compute_get_floating_object_min_z_index(doc_id: string, sheet_id: any): number;

export function compute_get_floating_object_typed(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_get_floating_objects_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_floating_objects_in_z_order(doc_id: string, sheet_id: any): any;

export function compute_get_format_categories_2d(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_formula(doc_id: string, cell_id: any): any;

export function compute_get_formula_reference_diagnostics(doc_id: string, options: any): any;

export function compute_get_frozen_panes_query(doc_id: string, sheet_id: any): any;

export function compute_get_group_in_sheet(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_groups(doc_id: string, sheet_id: any, axis: string): any;

export function compute_get_hf_images(doc_id: string, sheet_id: any): any;

export function compute_get_hidden_columns(doc_id: string, sheet_id: any): any;

export function compute_get_hidden_rows(doc_id: string, sheet_id: any): any;

export function compute_get_hidden_sheet_ids(doc_id: string): any;

export function compute_get_hyperlink(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_hyperlinks(doc_id: string, sheet_id: any): any;

export function compute_get_icon_set_preset_names(): any;

export function compute_get_icon_set_presets(doc_id: string): any;

export function compute_get_import_diagnostics(doc_id: string): any;

export function compute_get_max_outline_level(doc_id: string, sheet_id: any, axis: string): number;

export function compute_get_max_z_index(doc_id: string, sheet_id: any): number;

export function compute_get_max_z_index_all(doc_id: string, sheet_id: any): number;

export function compute_get_merge_at_cell_query(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_merge_at_cell_spatial(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_merges_in_viewport_spatial(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_min_z_index(doc_id: string, sheet_id: any): number;

export function compute_get_min_z_index_all(doc_id: string, sheet_id: any): number;

export function compute_get_named_range_array_values(doc_id: string, name: string, current_sheet: any): any;

export function compute_get_named_range_by_id(doc_id: string, id: string): any;

export function compute_get_named_range_by_name(doc_id: string, name: string, scope: any): any;

export function compute_get_named_range_display_value(doc_id: string, name: string, current_sheet: any): any;

export function compute_get_named_range_type(doc_id: string, name: string, current_sheet: any): any;

export function compute_get_named_range_typed_value(doc_id: string, name: string, current_sheet: any): any;

export function compute_get_named_ranges_by_scope(doc_id: string, scope: any): any;

export function compute_get_note_count(doc_id: string, sheet_id: any): number;

export function compute_get_or_create_cell_id(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_outline_gutter_dimensions(doc_id: string, sheet_id: any, level_width: number, level_height: number): any;

export function compute_get_outline_level_buttons(doc_id: string, sheet_id: any): any;

export function compute_get_outline_render_data(doc_id: string, sheet_id: any, viewport: any): any;

export function compute_get_outline_symbols(doc_id: string, sheet_id: any, viewport: any): any;

export function compute_get_page_breaks(doc_id: string, sheet_id: any): any;

export function compute_get_precedents(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_print_area(doc_id: string, sheet_id: any): any;

export function compute_get_print_settings(doc_id: string, sheet_id: any): any;

export function compute_get_print_titles(doc_id: string, sheet_id: any): any;

export function compute_get_projection_range(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_projection_source(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_range_schema(doc_id: string, sheet_id: any, schema_id: string): any;

export function compute_get_range_schemas_for_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_range_values_2d(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_range_with_identity(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_raw_cell_data(doc_id: string, sheet_id: any, row: number, col: number, include_formula: boolean): any;

export function compute_get_raw_value(doc_id: string, sheet_id: any, row: number, col: number): string;

export function compute_get_registered_viewports(doc_id: string): any;

export function compute_get_resolved_format(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_row_at_pixel(doc_id: string, sheet_id: any, y: number): number;

export function compute_get_row_formats(doc_id: string, sheet_id: any, rows: any): any;

export function compute_get_row_height_from_index(doc_id: string, sheet_id: any, row: number): number;

export function compute_get_row_height_query(doc_id: string, sheet_id: any, row: number): number;

export function compute_get_row_heights_batch(doc_id: string, sheet_id: any, start_row: number, end_row: number): any;

export function compute_get_row_outline_levels(doc_id: string, sheet_id: any, start_row: number, end_row: number): any;

export function compute_get_row_position(doc_id: string, sheet_id: any, row: number): number;

export function compute_get_runtime_diagnostics(doc_id: string, options: any): any;

export function compute_get_scroll_position_query(doc_id: string, sheet_id: any): any;

export function compute_get_selection_aggregates(doc_id: string, sheet_id: any, ranges: any): any;

export function compute_get_sheet_grouping_config(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_index(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_meta(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_name(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_order(doc_id: string): any;

export function compute_get_sheet_protection_config(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_settings(doc_id: string, sheet_id: any): any;

export function compute_get_sheet_visibility(doc_id: string, sheet_id: any): string;

export function compute_get_slicer_items_from_cache(doc_id: string, cache: any): any;

export function compute_get_slicer_state(doc_id: string, sheet_id: any, slicer_id: string): any;

export function compute_get_slicer_style(doc_id: string, name: string): any;

export function compute_get_slicer_style_count(doc_id: string): number;

export function compute_get_sparkline(doc_id: string, sheet_id: any, sparkline_id: string): any;

export function compute_get_sparkline_at_cell(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_sparkline_group(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_get_sparkline_groups_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_sparklines_in_sheet(doc_id: string, sheet_id: any): any;

export function compute_get_split_config(doc_id: string, sheet_id: any): any;

export function compute_get_subtotal_config(doc_id: string, sheet_id: any): any;

export function compute_get_tab_color_query(doc_id: string, sheet_id: any): any;

export function compute_get_table_annotation(doc_id: string, table_ref: string): any;

export function compute_get_table_at_cell(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_table_by_name(doc_id: string, table_name: string): any;

export function compute_get_table_filter(doc_id: string, sheet_id: any, table_id: string): any;

export function compute_get_table_hit_region(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_transferable_format(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_get_undo_state(doc_id: string): any;

export function compute_get_unique_column_values(doc_id: string, sheet_id: any, filter_id: string, header_col: number): any;

export function compute_get_value_for_editing(doc_id: string, sheet_id: any, row: number, col: number): string;

export function compute_get_value_types_2d(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_view_options_query(doc_id: string, sheet_id: any): any;

export function compute_get_viewport_binary(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, show_formulas: boolean): Uint8Array;

export function compute_get_viewport_binary_delta(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, show_formulas: boolean): Uint8Array;

export function compute_get_viewport_projection_data(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_get_visible_named_ranges(doc_id: string): any;

export function compute_get_visible_sheet_ids(doc_id: string): any;

export function compute_get_workbook_protection_options(doc_id: string): any;

export function compute_get_workbook_setting(doc_id: string, key: string): any;

export function compute_get_workbook_settings(doc_id: string): any;

export function compute_get_workbook_theme(doc_id: string): any;

export function compute_goal_seek(doc_id: string, params: any): any;

export function compute_group_columns(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_group_rows(doc_id: string, sheet_id: any, start_row: number, end_row: number): any;

export function compute_has_cf_for_cell(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_has_comments(doc_id: string, sheet_id: any, cell_id: string): boolean;

export function compute_has_comments_by_position(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_has_sheet_protection_password(doc_id: string, sheet_id: any): boolean;

export function compute_has_sparkline(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_has_workbook_protection_password(doc_id: string): boolean;

export function compute_hide_columns(doc_id: string, sheet_id: any, cols: any): any;

export function compute_hide_rows(doc_id: string, sheet_id: any, rows: any): any;

export function compute_import_from_csv_bytes(doc_id: string, csv_data: Uint8Array, options: any): any;

export function compute_import_from_xlsx_bytes(doc_id: string, xlsx_data: Uint8Array, do_recalc: boolean): any;

export function compute_import_from_xlsx_bytes_deferred(doc_id: string, xlsx_data: Uint8Array): any;

export function compute_import_named_ranges(doc_id: string, names: any): any;

export function compute_import_sheets_from_xlsx(doc_id: string, xlsx_data: Uint8Array, sheet_names: any, insert_position: any): any;

export function compute_import_values(doc_id: string, sheet_id: any, updates: any): any;

export function compute_init(doc_id: string, snapshot: any, layout_metrics: any): any;

export function compute_init_from_yrs_state(doc_id: string, state: Uint8Array, layout_metrics: any): any;

export function compute_insert_cells_with_shift(doc_id: string, sheet_id: any, row: number, col: number, row_count: number, col_count: number, shift_right: boolean): any;

export function compute_is_chart_linked_to_table(doc_id: string, sheet_id: any, chart_id: string): boolean;

export function compute_is_col_hidden_query(doc_id: string, sheet_id: any, col: number): boolean;

export function compute_is_column_visible_by_groups(doc_id: string, sheet_id: any, col: number): boolean;

export function compute_is_iterative_calculation_enabled(doc_id: string): boolean;

export function compute_is_merge_origin(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_is_projected_position(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_is_projection_source(doc_id: string, sheet_id: any, row: number, col: number): boolean;

export function compute_is_row_hidden_query(doc_id: string, sheet_id: any, row: number): boolean;

export function compute_is_row_visible_by_groups(doc_id: string, sheet_id: any, row: number): boolean;

export function compute_is_sheet_calculation_enabled(doc_id: string, sheet_id: any): boolean;

export function compute_is_sheet_hidden(doc_id: string, sheet_id: any): boolean;

export function compute_is_sheet_protected(doc_id: string, sheet_id: any): boolean;

export function compute_is_slicer_column_connected(doc_id: string, source_column_id: string, table_columns: any): boolean;

export function compute_is_workbook_operation_allowed(doc_id: string, operation: any): boolean;

export function compute_is_workbook_protected(doc_id: string): boolean;

export function compute_link_chart_to_table(doc_id: string, sheet_id: any, chart_id: string, table_id: string): any;

export function compute_list_cell_annotations(doc_id: string, sheet_id: any): any;

export function compute_list_custom_settings(doc_id: string): any;

export function compute_list_slicer_styles(doc_id: string): any;

export function compute_list_table_annotations(doc_id: string): any;

export function compute_make_principal(doc_id: string, tags: any): any;

export function compute_map_slicer_disconnection_reason(doc_id: string, reason: string): any;

export function compute_map_slicer_invalidation_reason(doc_id: string, reason: string): any;

export function compute_merge_across(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_merge_and_center(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_merge_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_move_floating_object_typed(doc_id: string, sheet_id: any, object_id: string, target: any): any;

export function compute_move_sheet(doc_id: string, sheet_id: any, new_index: number): any;

export function compute_named_range_count(doc_id: string): number;

export function compute_named_range_exists(doc_id: string, name: string, scope: any): boolean;

export function compute_parse_cell_ref(doc_id: string, cell_str: string): any;

export function compute_parse_date_input(doc_id: string, text: string): any;

export function compute_parse_range_ref(doc_id: string, range_str: string): any;

export function compute_patch_borders(doc_id: string, sheet_id: any, operations: any): any;

export function compute_patch_cell_properties_batch(doc_id: string, sheet_id: any, updates: any): any;

export function compute_patch_col_format(doc_id: string, sheet_id: any, col: number, format: any, clear_fields: any): any;

export function compute_patch_col_formats(doc_id: string, sheet_id: any, updates: any): any;

export function compute_patch_format_for_ranges(doc_id: string, sheet_id: any, ranges: any, format: any, clear_fields: any): any;

export function compute_patch_row_format(doc_id: string, sheet_id: any, row: number, format: any, clear_fields: any): any;

export function compute_patch_row_formats(doc_id: string, sheet_id: any, updates: any): any;

export function compute_patch_workbook_settings(doc_id: string, patch: any): any;

export function compute_pivot_compute_from_source(doc_id: string, sheet_id: any, pivot_id: string, expansion_state: any): any;

export function compute_pivot_create(doc_id: string, config: any): any;

export function compute_pivot_create_with_sheet(doc_id: string, sheet_name: string, config: any, options: any): any;

export function compute_pivot_delete(doc_id: string, sheet_id: any, pivot_id: string): any;

export function compute_pivot_get(doc_id: string, sheet_id: any, pivot_id: string): any;

export function compute_pivot_get_all(doc_id: string, sheet_id: any): any;

export function compute_pivot_get_all_items(doc_id: string, sheet_id: any, pivot_id: string, expansion_state: any): any;

export function compute_pivot_get_imported_view_records(doc_id: string, sheet_id: any): any;

export function compute_pivot_materialize(doc_id: string, sheet_id: any, pivot_id: string, expansion_state: any): any;

export function compute_pivot_materialize_mutation(doc_id: string, sheet_id: any, pivot_id: string, expansion_state: any): any;

export function compute_pivot_register_def(doc_id: string, sheet_id: any, pivot_id: string, total_rows: number, total_cols: number, first_data_row: number, first_data_col: number): any;

export function compute_pivot_unregister_def(doc_id: string, sheet_id: any, pivot_name: string): any;

export function compute_pivot_update(doc_id: string, sheet_id: any, pivot_id: string, config: any): any;

export function compute_pivot_update_and_materialize(doc_id: string, sheet_id: any, pivot_id: string, config: any, expansion_state: any): any;

export function compute_prepare_date_value(year: number, month: number, day: number, existing_format: any): any;

export function compute_prepare_time_value(hours: number, minutes: number, seconds: number, existing_format: any): any;

export function compute_preview_text_to_columns(doc_id: string, sheet_id: any, source_start_row: number, source_end_row: number, source_col: number, options: any, max_preview_rows: number): any;

export function compute_protect_sheet(doc_id: string, sheet_id: any, password_hash: any): any;

export function compute_protect_sheet_with_options(doc_id: string, sheet_id: any, password_hash: any, options: any): any;

export function compute_protect_workbook(doc_id: string, password_hash: any, options: any): any;

export function compute_query_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_query_range_properties(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_query_ranges(doc_id: string, requests: any): any;

export function compute_reapply_filter(doc_id: string, sheet_id: any, filter_id: string): any;

export function compute_redo(doc_id: string): any;

export function compute_regex_search(doc_id: string, sheet_id: any, options: any): any;

export function compute_regex_search_all_sheets(doc_id: string, options: any): any;

export function compute_register_viewport(doc_id: string, viewport_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_relocate_cells(doc_id: string, sheet_id: any, src_start_row: number, src_start_col: number, src_end_row: number, src_end_col: number, target_row: number, target_col: number): any;

export function compute_relocate_cells_yrs(doc_id: string, source_sheet_id: any, src_start_row: number, src_start_col: number, src_end_row: number, src_end_col: number, target_sheet_id: any, target_row: number, target_col: number): any;

export function compute_remove_binding(doc_id: string, sheet_id: any, binding_id: string): any;

export function compute_remove_bindings_for_connection(doc_id: string, connection_id: string): any;

export function compute_remove_calculated_column(doc_id: string, table_name: string, column_index: number): any;

export function compute_remove_cell_annotation_by_position(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_remove_compute_sheet(doc_id: string, sheet_id: any): any;

export function compute_remove_duplicates(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, columns: any, has_headers: boolean): any;

export function compute_remove_hf_image(doc_id: string, sheet_id: any, position: any): any;

export function compute_remove_horizontal_page_break(doc_id: string, sheet_id: any, row: number): any;

export function compute_remove_hyperlink(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_remove_named_range(doc_id: string, name: string): any;

export function compute_remove_named_range_by_id(doc_id: string, id: string): any;

export function compute_remove_named_ranges_by_scope(doc_id: string, scope: any): any;

export function compute_remove_scenario(doc_id: string, scenario_id: string): any;

export function compute_remove_schema(doc_id: string, sheet_id: string, column: number, version: number): boolean;

export function compute_remove_subtotals(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_remove_table_annotation(doc_id: string, table_ref: string): any;

export function compute_remove_table_column(doc_id: string, table_name: string, column_index: number): any;

export function compute_remove_table_data_row(doc_id: string, table_name: string, relative_row: number): any;

export function compute_remove_vertical_page_break(doc_id: string, sheet_id: any, col: number): any;

export function compute_rename_compute_sheet(doc_id: string, sheet_id: any, name: string): any;

export function compute_rename_table(doc_id: string, old_name: string, new_name: string): any;

export function compute_rename_table_column(doc_id: string, table_name: string, column_index: number, new_column_name: string): any;

export function compute_reorder_cf_rules(doc_id: string, sheet_id: any, rule_ids: any): any;

export function compute_reorder_sheets(doc_id: string, new_order: any): any;

export function compute_replace_all_in_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, text: string, replacement: string, options: any): any;

export function compute_reset_sheet_viewports(doc_id: string, sheet_id: any): any;

export function compute_reset_viewport_state(doc_id: string, sheet_id: any): any;

export function compute_reset_workbook_settings(doc_id: string): any;

export function compute_resize_floating_object_typed(doc_id: string, sheet_id: any, object_id: string, config: any): any;

export function compute_resize_table(doc_id: string, table_name: string, new_start_row: number, new_start_col: number, new_end_row: number, new_end_col: number): any;

export function compute_resolve_cell_positions(doc_id: string, cell_id_hexes: any): any;

export function compute_resolve_named_range(doc_id: string, name: string, current_sheet: any): any;

export function compute_resolve_table_format_at_cell(doc_id: string, sheet_id: any, row: number, col: number): any;

export function compute_restore_scenario(doc_id: string, baseline_id: string): any;

export function compute_rotate_floating_object_typed(doc_id: string, sheet_id: any, object_id: string, rotation: number): any;

export function compute_schema_infer_column(values: any): any;

export function compute_schema_infer_type(value: any): any;

export function compute_schema_resolve_editor(input: any): any;

export function compute_schema_validate(value: any, schema: any): any;

export function compute_security_active(doc_id: string): boolean;

export function compute_semantic_workbook_state_envelope(doc_id: string): any;

export function compute_send_chart_backward(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_send_chart_to_back(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_send_floating_object_backward(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_send_floating_object_to_back(doc_id: string, sheet_id: any, object_id: string): any;

export function compute_set_active_principal(doc_id: string, tags: any): void;

export function compute_set_active_scenario(doc_id: string, scenario_id: any): any;

export function compute_set_array_formula(doc_id: string, sheet_id: any, top_row: number, left_col: number, bottom_row: number, right_col: number, formula: string): any;

export function compute_set_calculated_column_formula(doc_id: string, table_name: string, column_index: number, formula: string): any;

export function compute_set_calculation_mode(doc_id: string, mode: string): any;

export function compute_set_calculation_settings(doc_id: string, settings: any): any;

export function compute_set_cell(doc_id: string, sheet_id: any, cell_id: any, row: number, col: number, input: any): any;

export function compute_set_cell_annotation_by_position(doc_id: string, sheet_id: any, row: number, col: number, text: string): any;

export function compute_set_cell_binary(doc_id: string, sheet_id: any, cell_id: any, row: number, col: number, input: any): any;

export function compute_set_cell_format(doc_id: string, sheet_id: any, cell_id: any, format: any): any;

export function compute_set_cell_properties_batch(doc_id: string, sheet_id: any, updates: any): any;

export function compute_set_cell_value_as_text(doc_id: string, sheet_id: any, row: number, col: number, value: string): any;

export function compute_set_cell_value_parsed(doc_id: string, sheet_id: any, row: number, col: number, raw_input: string): any;

export function compute_set_cell_values_parsed(doc_id: string, sheet_id: any, updates: any): any;

export function compute_set_cells_batch(doc_id: string, sheet_id: any, cells: any): any;

export function compute_set_col_format(doc_id: string, sheet_id: any, col: number, format: any): any;

export function compute_set_col_format_range(doc_id: string, sheet_id: any, start_col: number, end_col: number, format: any): any;

export function compute_set_col_formats(doc_id: string, sheet_id: any, updates: any): any;

export function compute_set_col_width(doc_id: string, sheet_id: any, col: number, width_px: number): any;

export function compute_set_col_width_chars(doc_id: string, sheet_id: any, col: number, width_chars: number): any;

export function compute_set_col_widths(doc_id: string, sheet_id: any, widths: any): any;

export function compute_set_col_widths_chars(doc_id: string, sheet_id: any, widths: any): any;

export function compute_set_column_filter(doc_id: string, sheet_id: any, filter_id: string, header_col: number, criteria: any): any;

export function compute_set_column_schema(doc_id: string, sheet_id: any, col_index: number, schema: any): any;

export function compute_set_convergence_threshold(doc_id: string, threshold: number): any;

export function compute_set_culture(doc_id: string, culture: string): any;

export function compute_set_current_time(timestamp_serial: number): void;

export function compute_set_custom_setting(doc_id: string, key: string, value: any): any;

export function compute_set_date_value(doc_id: string, sheet_id: any, row: number, col: number, year: number, month: number, day: number): any;

export function compute_set_default_pivot_table_style(doc_id: string, style_id: any): any;

export function compute_set_default_slicer_style(doc_id: string, style_id: any): any;

export function compute_set_default_table_style_id(doc_id: string, style_id: any): any;

export function compute_set_document_properties(doc_id: string, props: any): any;

export function compute_set_filter_sort_state(doc_id: string, sheet_id: any, filter_id: string, sort_state: any): any;

export function compute_set_floating_object(doc_id: string, sheet_id: any, object_id: string, json: any): any;

export function compute_set_floating_object_group(doc_id: string, sheet_id: any, group_id: string, json: any): any;

export function compute_set_format_for_ranges(doc_id: string, sheet_id: any, ranges: any, format: any): any;

export function compute_set_format_for_ranges_ui_state(doc_id: string, sheet_id: any, ranges: any, format: any): any;

export function compute_set_frozen_panes(doc_id: string, sheet_id: any, rows: number, cols: number): any;

export function compute_set_group_collapsed(doc_id: string, sheet_id: any, group_id: string, collapsed: boolean): any;

export function compute_set_hf_image(doc_id: string, sheet_id: any, info: any): any;

export function compute_set_hyperlink(doc_id: string, sheet_id: any, row: number, col: number, url: string): any;

export function compute_set_iterative_calculation(doc_id: string, enabled: boolean): any;

export function compute_set_iterative_calculation_enabled(doc_id: string, enabled: boolean): any;

export function compute_set_level_collapsed(doc_id: string, sheet_id: any, axis: string, level: number, collapsed: boolean): any;

export function compute_set_max_iterations(doc_id: string, n: number): any;

export function compute_set_named_range(doc_id: string, name: string, def: any): any;

export function compute_set_note_dimensions(doc_id: string, sheet_id: any, comment_id: string, height: any, width: any): any;

export function compute_set_note_visible(doc_id: string, sheet_id: any, comment_id: string, visible: boolean): any;

export function compute_set_outline_settings(doc_id: string, sheet_id: any, settings: any): any;

export function compute_set_print_area(doc_id: string, sheet_id: any, area: any): any;

export function compute_set_print_settings(doc_id: string, sheet_id: any, settings: any): any;

export function compute_set_print_titles(doc_id: string, sheet_id: any, titles: any): any;

export function compute_set_range_schema(doc_id: string, sheet_id: any, schema: any): any;

export function compute_set_row_format(doc_id: string, sheet_id: any, row: number, format: any): any;

export function compute_set_row_formats(doc_id: string, sheet_id: any, updates: any): any;

export function compute_set_row_height(doc_id: string, sheet_id: any, row: number, height_px: number): any;

export function compute_set_schema_map(doc_id: string, entries: any, version: number): void;

export function compute_set_scroll_position(doc_id: string, sheet_id: any, top_row: number, left_col: number): any;

export function compute_set_sheet_enable_calculation(doc_id: string, sheet_id: any, enabled: boolean): any;

export function compute_set_sheet_hidden(doc_id: string, sheet_id: any, hidden: boolean): any;

export function compute_set_sheet_protection_options(doc_id: string, sheet_id: any, options: any): any;

export function compute_set_sheet_setting(doc_id: string, sheet_id: any, key: string, value: string): any;

export function compute_set_sheet_visibility(doc_id: string, sheet_id: any, state: string): any;

export function compute_set_slicer_selection(doc_id: string, sheet_id: any, slicer_id: string, values: any): any;

export function compute_set_split_config(doc_id: string, sheet_id: any, config: any): any;

export function compute_set_tab_color(doc_id: string, sheet_id: any, color: any): any;

export function compute_set_table_annotation(doc_id: string, table_ref: string, text: string): any;

export function compute_set_table_auto_calculated_columns(doc_id: string, table_name: string, enabled: boolean): any;

export function compute_set_table_auto_expand(doc_id: string, table_name: string, enabled: boolean): any;

export function compute_set_table_bool_option(doc_id: string, table_name: string, option: string, value: boolean): any;

export function compute_set_table_style(doc_id: string, table_name: string, style_name: string): any;

export function compute_set_table_totals_function(doc_id: string, table_name: string, column_id: string, func: any): any;

export function compute_set_thread_resolved(doc_id: string, sheet_id: any, cell_id: string, resolved: boolean): any;

export function compute_set_time_value(doc_id: string, sheet_id: any, row: number, col: number, hours: number, minutes: number, seconds: number): any;

export function compute_set_use_precision_as_displayed(doc_id: string, enabled: boolean): any;

export function compute_set_view_option(doc_id: string, sheet_id: any, key: string, value: boolean): any;

export function compute_set_workbook_setting(doc_id: string, key: string, value: any): any;

export function compute_set_workbook_settings(doc_id: string, settings: any): any;

export function compute_set_workbook_theme(doc_id: string, theme: any): any;

export function compute_settle_for_mirror(doc_id: string): any;

export function compute_should_render_outlines(doc_id: string, sheet_id: any): boolean;

export function compute_sign_check(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, options: any): any;

export function compute_sign_check_a1(doc_id: string, sheet_id: any, range_a1: any, options: any): any;

export function compute_solve(doc_id: string, params: any): any;

export function compute_sort_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number, options: any): any;

export function compute_stringify_cell_ref(doc_id: string, cell: any): any;

export function compute_stringify_range_ref(doc_id: string, range: any): any;

export function compute_structure_change(doc_id: string, sheet_id: any, change: any): any;

export function compute_sync_full_state(doc_id: string): Uint8Array;

export function compute_text_to_columns(doc_id: string, sheet_id: any, start_row: number, end_row: number, source_col: number, dest_row: number, dest_col: number, options: any): any;

export function compute_text_to_columns_simple(doc_id: string, sheet_id: any, start_row: number, end_row: number, source_col: number, dest_row: number, dest_col: number, delimiter: string, custom_delimiter: any, treat_consecutive_as_one: boolean, text_qualifier: string): any;

export function compute_to_a1_display(doc_id: string, sheet_id: any, formula: any): string;

export function compute_to_a1_display_qualified(doc_id: string, sheet_id: any, formula: any): string;

export function compute_to_identity_formula(doc_id: string, sheet_id: any, formula_a1: string): any;

export function compute_toggle_banded_cols(doc_id: string, table_name: string): any;

export function compute_toggle_banded_rows(doc_id: string, table_name: string): any;

export function compute_toggle_format_property(doc_id: string, sheet_id: any, ranges: any, property: string, active_row: number, active_col: number): any;

export function compute_toggle_group_collapsed(doc_id: string, sheet_id: any, group_id: string): any;

export function compute_toggle_header_row(doc_id: string, table_name: string): any;

export function compute_toggle_slicer_item(doc_id: string, sheet_id: any, slicer_id: string, value: any): any;

export function compute_toggle_totals_row(doc_id: string, table_name: string): any;

export function compute_undo(doc_id: string): any;

export function compute_ungroup_columns(doc_id: string, sheet_id: any, start_col: number, end_col: number): any;

export function compute_ungroup_rows(doc_id: string, sheet_id: any, start_row: number, end_row: number): any;

export function compute_unhide_columns(doc_id: string, sheet_id: any, cols: any): any;

export function compute_unhide_rows(doc_id: string, sheet_id: any, rows: any): any;

export function compute_unlink_chart_from_table(doc_id: string, sheet_id: any, chart_id: string): any;

export function compute_unmerge_range(doc_id: string, sheet_id: any, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_unprotect_sheet(doc_id: string, sheet_id: any, password_hash: any): any;

export function compute_unprotect_workbook(doc_id: string, password_hash: any): any;

export function compute_unregister_viewport(doc_id: string, viewport_id: string): any;

export function compute_update_binding(doc_id: string, sheet_id: any, binding_id: string, updates: any): any;

export function compute_update_calculated_column(doc_id: string, table_name: string, column_index: number, formula: string): any;

export function compute_update_cell_position(doc_id: string, sheet_id: any, cell_id_hex: string, new_row: number, new_col: number): any;

export function compute_update_cf_ranges(doc_id: string, sheet_id: any, format_id: string, new_ranges: any): any;

export function compute_update_cf_rule(doc_id: string, sheet_id: any, rule_id: string, updates: any): any;

export function compute_update_chart(doc_id: string, sheet_id: any, chart_id: string, updates: any): any;

export function compute_update_comment(doc_id: string, sheet_id: any, comment_id: string, text: string): any;

export function compute_update_comment_mentions(doc_id: string, sheet_id: any, comment_id: string, content: string, mentions: any): any;

export function compute_update_custom_cell_style(doc_id: string, id: string, style: any): any;

export function compute_update_custom_table_style(doc_id: string, style_name: string, style: any): any;

export function compute_update_floating_object(doc_id: string, sheet_id: any, object_id: string, updates: any): any;

export function compute_update_floating_object_group(doc_id: string, sheet_id: any, group_id: string, updates: any): any;

export function compute_update_named_range(doc_id: string, id: string, updates: any): any;

export function compute_update_range_schema(doc_id: string, sheet_id: any, schema_id: string, updates: any): any;

export function compute_update_refresh_metadata(doc_id: string, sheet_id: any, binding_id: string, last_refresh: bigint, last_row_count: number): any;

export function compute_update_rule_in_cf(doc_id: string, sheet_id: any, format_id: string, rule_id: string, updates: any): any;

export function compute_update_scenario(doc_id: string, scenario_id: string, input: any): any;

export function compute_update_schema(doc_id: string, sheet_id: string, column: number, schema: any, version: number): boolean;

export function compute_update_shape_style(doc_id: string, sheet_id: any, object_id: string, style: any): any;

export function compute_update_slicer_config(doc_id: string, sheet_id: any, slicer_id: string, update: any): any;

export function compute_update_sparkline(doc_id: string, sheet_id: any, sparkline_id: string, updates: any): any;

export function compute_update_viewport_bounds(doc_id: string, viewport_id: string, start_row: number, start_col: number, end_row: number, end_col: number): any;

export function compute_validate_and_clean_comments(doc_id: string, sheet_id: any): any;

export function compute_validate_and_clean_merges(doc_id: string, sheet_id: any): any;

export function compute_validate_cell_value(doc_id: string, sheet_id: any, row: number, col: number, value: string): any;

export function compute_validate_formula_circular_reference(doc_id: string, sheet_id: any, row: number, col: number, formula: string): any;

export function compute_validate_formula_syntax(doc_id: string, _sheet_id: any, formula: string): any;

export function compute_validate_named_range_name(doc_id: string, name: string, scope: any, exclude_id: any): any;

export function compute_wb_security_add_policy(doc_id: string, policy: any): any;

export function compute_wb_security_apply_template(doc_id: string, template: any): any;

export function compute_wb_security_drain_events(doc_id: string): any;

export function compute_wb_security_effective_access(doc_id: string, target: any, principal_tags: any): any;

export function compute_wb_security_explain_access(doc_id: string, target: any, principal_tags: any): any;

export function compute_wb_security_list_policies(doc_id: string): any;

export function compute_wb_security_remove_policy(doc_id: string, id: any): void;

export function compute_wb_security_remove_template(doc_id: string, template_id: string): void;

export function compute_wb_security_update_policy(doc_id: string, id: any, patch: any): void;

export function pivot_compute(config: any, data: any, expansion_state: any): any;

export function pivot_compute_with_show_values_as(config: any, data: any, expansion_state: any): any;

export function pivot_detect_fields(data: any): any;

export function pivot_drill_down(config: any, data: any, row_key: string, column_key: string): any;

export function pivot_validate_config(config: any): any;

export function table_add_column(table: any, name: string, position: any): any;

export function table_adjust_structured_ref(sref: any, change: any): any;

export function table_all_visible(count: number): any;

export function table_build_filter_dropdown(column_data: any, current_filter: any, row_visibility: any): any;

export function table_build_slicer_cache(slicer: any, column_data: any, row_visibility: any): any;

export function table_cell_value_key(value: any): string;

export function table_cell_values_equal(a: any, b: any): boolean;

export function table_clear_slicer_selection(slicer: any): any;

export function table_compare_values(a: any, b: any): number;

export function table_compose_bitmaps(bitmaps: any): Uint8Array;

export function table_compute_sort_order(specs: any, data: any, total_rows: number): any;

export function table_create_row_visibility(bitmap: Uint8Array): any;

export function table_create_table(name: string, sheet_id: string, range: any, header_values: any, id: any, style: any): any;

export function table_evaluate_column_filter(criteria: any, column_data: any): Uint8Array;

export function table_evaluate_top_bottom(filter: any, column_data: any): Uint8Array;

export function table_format_cell_display(value: any): string;

export function table_format_structured_ref(sref: any): string;

export function table_generate_table_name(existing_names: any): string;

export function table_get_built_in_styles(): any;

export function table_get_column_at_grid_col(table: any, grid_col: number): any;

export function table_get_column_by_id(table: any, id: string): any;

export function table_get_column_by_name(table: any, name: string): any;

export function table_get_column_data_range(table: any, column_id: string): any;

export function table_get_data_range(table: any): any;

export function table_get_header_range(table: any): any;

export function table_get_totals_formula(func: any, column_name: string): string;

export function table_get_totals_range(table: any): any;

export function table_is_in_data_range(table: any, row: number, col: number): boolean;

export function table_is_in_header_row(table: any, row: number): boolean;

export function table_is_in_table(table: any, row: number, col: number): boolean;

export function table_is_in_totals_row(table: any, row: number): boolean;

export function table_parse_structured_ref(input: string): any;

export function table_ranges_overlap(a: any, b: any): boolean;

export function table_remove_column(table: any, column_id: string): any;

export function table_rename_column(table: any, column_id: string, new_name: string): any;

export function table_resize_table(table: any, new_range: any): any;

export function table_resolve_cell_format(table: any, row: number, col: number): any;

export function table_resolve_dynamic_filter(filter: any, column_data: any): any;

export function table_resolve_structured_ref(sref: any, tables: any, current_row: any): any;

export function table_select_all_slicer_values(slicer: any, cache: any): any;

export function table_select_slicer_values(slicer: any, values: any): any;

export function table_set_slicer_sort_order(slicer: any, order: any): any;

export function table_set_table_option(table: any, option: any, value: boolean): any;

export function table_set_totals_function(table: any, column_id: string, func: any): any;

export function table_slicer_to_filter_criteria(slicer: any): any;

export function table_toggle_slicer_value(slicer: any, value: any): any;

export function table_toggle_totals_row(table: any): any;

export function table_validate_table_name(name: string, existing_names: any): any;

export function table_value_in_list(value: any, list: any): boolean;

export function versioning_diff_semantic_workbook_states(before: any, after: any): any;

/**
 * Initialize the WASM module — sets the panic hook and tracing subscriber.
 * Called automatically by the generated WASM glue code on module init.
 */
export function wasm_start(): void;

export function xlsx_parse_lazy(xlsx_data: Uint8Array): any;

export function xlsx_parse_lazy_with_mode(xlsx_data: Uint8Array, mode: number): any;

export function xlsx_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly chart_apply_transforms: (a: number, b: number, c: number) => void;
    readonly chart_compute_bins: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly chart_compute_density: (a: number, b: number, c: number, d: number) => void;
    readonly chart_compute_regression: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly chart_compute_stacking: (a: number, b: number, c: number) => void;
    readonly chart_compute_statistics: (a: number, b: number) => void;
    readonly compute_active_principal: (a: number, b: number, c: number) => void;
    readonly compute_add_calculated_column: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_add_cf_rule: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_add_comment: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => void;
    readonly compute_add_comment_by_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => void;
    readonly compute_add_compute_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_add_horizontal_page_break: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_add_rule_to_cf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_add_slicer_style: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_add_sparkline: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_add_sparkline_group: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_add_table_column: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_add_table_data_row: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_add_vertical_page_break: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_apply_advanced_filter: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_apply_auto_expansion: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_apply_calculated_formulas_to_row: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_apply_changes: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_apply_filter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_apply_scenario: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_apply_sync_update: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_auto_fill: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_auto_fill_preview: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_auto_fit_column_and_set: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_auto_fit_columns_and_set: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_auto_fit_rows_and_set: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_auto_outline: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_batch_clear_cells: (a: number, b: number, c: number, d: number) => void;
    readonly compute_batch_set_cells: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_batch_set_cells_by_position: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_begin_undo_group: (a: number, b: number, c: number) => void;
    readonly compute_bring_chart_forward: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_bring_chart_to_front: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_bring_floating_object_forward: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_bring_floating_object_to_front: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_can_do_structure_op: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_can_edit_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_can_redo: (a: number, b: number, c: number) => void;
    readonly compute_can_undo: (a: number, b: number, c: number) => void;
    readonly compute_capture_screenshot: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => void;
    readonly compute_cf_intersect_ranges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_cf_is_valid_range: (a: number, b: number, c: number, d: number) => void;
    readonly compute_cf_range_contains: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_cf_ranges_overlap: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_cf_subtract_range: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_check_merge_data_loss: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_check_sort_range_merges: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_classify_value_type: (a: number, b: number) => void;
    readonly compute_clear_all_column_filters: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_clear_all_comments: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_all_filters: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_all_grouping: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_all_merges: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_all_page_breaks: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_cell_format: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_clear_cf_formats_for_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_col_format: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_clear_column_filter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_clear_column_grouping: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_clear_column_schema: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_clear_format_for_ranges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_clear_hyperlinks_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_clear_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_clear_range_and_return_ids: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_clear_range_by_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_clear_range_with_mode: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_clear_row_grouping: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_clear_schemas: (a: number, b: number, c: number) => void;
    readonly compute_clear_slicer_selection: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_clear_sparklines_for_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_clear_sparklines_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_collapse_all_groups: (a: number, b: number, c: number, d: number) => void;
    readonly compute_complete_deferred_hydration: (a: number, b: number, c: number) => void;
    readonly compute_compute_all_object_bounds: (a: number, b: number, c: number, d: number) => void;
    readonly compute_compute_dynamic_filter_serial_range: (a: number, b: number, c: number, d: number) => void;
    readonly compute_convert_note_to_thread: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_convert_table_to_range: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_copy_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly compute_copy_sheet: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_count_visible_sheets: (a: number, b: number, c: number) => void;
    readonly compute_create_binding: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_chart: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_custom_cell_style: (a: number, b: number, c: number, d: number) => void;
    readonly compute_create_custom_table_style: (a: number, b: number, c: number, d: number) => void;
    readonly compute_create_data_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_create_default_sheet: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_default_sheet_with_default_col_width: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_create_filter: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_floating_object: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_floating_object_group: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_named_range: (a: number, b: number, c: number, d: number) => void;
    readonly compute_create_scenario: (a: number, b: number, c: number, d: number) => void;
    readonly compute_create_shape: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_sheet: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_sheet_with_default_col_width: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_create_slicer: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_create_subtotals: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_create_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly compute_create_table_lifecycle: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly compute_current_state_vector: (a: number, b: number, c: number) => void;
    readonly compute_data_table: (a: number, b: number, c: number, d: number) => void;
    readonly compute_delete_cells_with_shift: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_delete_cf_rule: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_chart: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_comment: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_comments_for_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_comments_for_cell_by_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_custom_cell_style: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_delete_custom_table_style: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_delete_filter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_floating_object: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_floating_object_group: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_range_schema: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_rule_from_cf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_delete_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_delete_slicer: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_slicer_style: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_delete_slicers: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_delete_sparkline: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_delete_sparkline_group: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_delete_table: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_destroy: (a: number, b: number, c: number) => void;
    readonly compute_detect_auto_expansion: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_detect_format_type: (a: number, b: number, c: number) => void;
    readonly compute_drain_pending_updates: (a: number, b: number, c: number) => void;
    readonly compute_duplicate_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_duplicate_slicer_style: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_encode_diff: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_encode_state_vector: (a: number, b: number, c: number) => void;
    readonly compute_end_undo_group: (a: number, b: number, c: number) => void;
    readonly compute_eval_cf: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_evaluate_expression: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_expand_all_groups: (a: number, b: number, c: number, d: number) => void;
    readonly compute_export_to_xlsx_bytes: (a: number, b: number, c: number) => void;
    readonly compute_export_to_xlsx_bytes_context_stripped: (a: number, b: number, c: number) => void;
    readonly compute_find_all_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_find_cells_by_formula: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_find_cells_by_value: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_find_connectors_for_shape: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_find_data_edge: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_find_disconnected_slicers: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_find_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_find_last_column: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_find_last_row: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_find_slicers_for_table: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_flash_fill: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_flip_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_flush_undo_capture: (a: number, b: number, c: number) => void;
    readonly compute_format_cell_value_for_display: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_format_values: (a: number, b: number, c: number, d: number) => void;
    readonly compute_freeze_columns: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_freeze_rows: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_full_recalc: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_active_cell: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_active_filter_count: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_active_filters: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_active_scenario_state: (a: number, b: number, c: number) => void;
    readonly compute_get_affected_columns_by_group: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_affected_rows_by_group: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_all_bindings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_cells_yrs: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_cf_rules: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_charts: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_column_schemas: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_comments: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_comments_workbook: (a: number, b: number, c: number) => void;
    readonly compute_get_all_custom_cell_styles: (a: number, b: number, c: number) => void;
    readonly compute_get_all_custom_table_styles: (a: number, b: number, c: number) => void;
    readonly compute_get_all_floating_object_groups_typed: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_floating_objects_typed: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_in_z_order: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_merges_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_named_ranges_wire: (a: number, b: number, c: number) => void;
    readonly compute_get_all_notes: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_pivot_tables_workbook: (a: number, b: number, c: number) => void;
    readonly compute_get_all_scenarios: (a: number, b: number, c: number) => void;
    readonly compute_get_all_sheet_ids: (a: number, b: number, c: number) => void;
    readonly compute_get_all_slicers: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_slicers_workbook: (a: number, b: number, c: number) => void;
    readonly compute_get_all_tables_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_all_tables_workbook: (a: number, b: number, c: number) => void;
    readonly compute_get_binding: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_bindings_for_connection: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_calc_mode: (a: number, b: number, c: number) => void;
    readonly compute_get_calculation_settings: (a: number, b: number, c: number) => void;
    readonly compute_get_cell_annotation_by_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_count: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_cell_data: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_data_by_id_hex: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_format: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_get_cell_format_with_cf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_get_cell_id_at: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_id_at_yrs: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_info: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cell_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_cells_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_cells_in_range_yrs: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_cf_preset_by_id: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_cf_presets: (a: number) => void;
    readonly compute_get_cf_rules_for_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_chart: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_charts_in_z_order: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_charts_linked_to_table: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_col_at_pixel: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_position: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_width_chars_query: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_width_from_index: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_width_query: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_col_widths_batch: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_col_widths_batch_chars: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_color_scale_presets: (a: number) => void;
    readonly compute_get_column_outline_levels: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_column_schema: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_comment: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_comment_count: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_comment_thread: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_comments_for_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_comments_for_cell_by_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_conditional_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_current_region: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_custom_setting: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_data_bar_presets: (a: number) => void;
    readonly compute_get_data_bounds: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_data_bounds_for_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_get_default_col_width: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_default_col_width_chars: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_default_font: (a: number, b: number, c: number) => void;
    readonly compute_get_default_pivot_table_style: (a: number, b: number, c: number) => void;
    readonly compute_get_default_row_height: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_default_slicer_style: (a: number, b: number, c: number) => void;
    readonly compute_get_default_table_style_id: (a: number, b: number, c: number) => void;
    readonly compute_get_dependents: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_display_text_2d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_display_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_displayed_cell_properties: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_displayed_range_properties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_document_properties: (a: number, b: number, c: number) => void;
    readonly compute_get_effective_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_filter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_filter_count: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_filter_header_info: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_filter_hidden_rows: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_filter_sort_state: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_filtered_record_count: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_filters_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_first_sheet_id: (a: number, b: number, c: number) => void;
    readonly compute_get_floating_object: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_floating_object_group: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_floating_object_group_typed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_floating_object_groups_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_floating_object_max_z_index: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_floating_object_min_z_index: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_floating_objects_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_floating_objects_in_z_order: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_format_categories_2d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_formula: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_formula_reference_diagnostics: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_frozen_panes_query: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_group_in_sheet: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_groups: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_hf_images: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_hidden_columns: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_hidden_rows: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_hidden_sheet_ids: (a: number, b: number, c: number) => void;
    readonly compute_get_hyperlink: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_hyperlinks: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_icon_set_preset_names: (a: number) => void;
    readonly compute_get_icon_set_presets: (a: number, b: number, c: number) => void;
    readonly compute_get_import_diagnostics: (a: number, b: number, c: number) => void;
    readonly compute_get_max_outline_level: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_max_z_index: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_max_z_index_all: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_merge_at_cell_query: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_merge_at_cell_spatial: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_merges_in_viewport_spatial: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_min_z_index: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_min_z_index_all: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_named_range_array_values: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_named_range_by_id: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_named_range_by_name: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_named_range_display_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_named_range_type: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_named_range_typed_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_named_ranges_by_scope: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_note_count: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_or_create_cell_id: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_outline_gutter_dimensions: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_outline_level_buttons: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_outline_render_data: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_outline_symbols: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_page_breaks: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_precedents: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_print_area: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_print_settings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_print_titles: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_projection_range: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_projection_source: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_range_schema: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_range_schemas_for_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_range_values_2d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_range_with_identity: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_raw_cell_data: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_get_raw_value: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_registered_viewports: (a: number, b: number, c: number) => void;
    readonly compute_get_resolved_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_row_at_pixel: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_row_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_row_height_from_index: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_row_height_query: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_row_heights_batch: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_row_outline_levels: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_row_position: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_runtime_diagnostics: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_scroll_position_query: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_selection_aggregates: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_sheet_grouping_config: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_index: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_meta: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_name: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_order: (a: number, b: number, c: number) => void;
    readonly compute_get_sheet_protection_config: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_settings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sheet_visibility: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_slicer_items_from_cache: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_slicer_state: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_slicer_style: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_slicer_style_count: (a: number, b: number, c: number) => void;
    readonly compute_get_sparkline: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_sparkline_at_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_sparkline_group: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_sparkline_groups_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_sparklines_in_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_split_config: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_subtotal_config: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_tab_color_query: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_table_annotation: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_table_at_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_table_by_name: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_table_filter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_table_hit_region: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_transferable_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_undo_state: (a: number, b: number, c: number) => void;
    readonly compute_get_unique_column_values: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_get_value_for_editing: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_get_value_types_2d: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_view_options_query: (a: number, b: number, c: number, d: number) => void;
    readonly compute_get_viewport_binary: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_get_viewport_binary_delta: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_get_viewport_projection_data: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_get_visible_named_ranges: (a: number, b: number, c: number) => void;
    readonly compute_get_visible_sheet_ids: (a: number, b: number, c: number) => void;
    readonly compute_get_workbook_protection_options: (a: number, b: number, c: number) => void;
    readonly compute_get_workbook_setting: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_get_workbook_settings: (a: number, b: number, c: number) => void;
    readonly compute_get_workbook_theme: (a: number, b: number, c: number) => void;
    readonly compute_goal_seek: (a: number, b: number, c: number, d: number) => void;
    readonly compute_group_columns: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_group_rows: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_has_cf_for_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_has_comments: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_has_comments_by_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_has_sheet_protection_password: (a: number, b: number, c: number, d: number) => void;
    readonly compute_has_sparkline: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_has_workbook_protection_password: (a: number, b: number, c: number) => void;
    readonly compute_hide_columns: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_hide_rows: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_import_from_csv_bytes: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_import_from_xlsx_bytes: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_import_from_xlsx_bytes_deferred: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_import_named_ranges: (a: number, b: number, c: number, d: number) => void;
    readonly compute_import_sheets_from_xlsx: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_import_values: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_init: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_init_from_yrs_state: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_insert_cells_with_shift: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_is_chart_linked_to_table: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_is_col_hidden_query: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_is_column_visible_by_groups: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_is_iterative_calculation_enabled: (a: number, b: number, c: number) => void;
    readonly compute_is_merge_origin: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_is_projected_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_is_projection_source: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_is_row_hidden_query: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_is_row_visible_by_groups: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_is_sheet_calculation_enabled: (a: number, b: number, c: number, d: number) => void;
    readonly compute_is_sheet_hidden: (a: number, b: number, c: number, d: number) => void;
    readonly compute_is_sheet_protected: (a: number, b: number, c: number, d: number) => void;
    readonly compute_is_slicer_column_connected: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_is_workbook_operation_allowed: (a: number, b: number, c: number, d: number) => void;
    readonly compute_is_workbook_protected: (a: number, b: number, c: number) => void;
    readonly compute_link_chart_to_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_list_cell_annotations: (a: number, b: number, c: number, d: number) => void;
    readonly compute_list_custom_settings: (a: number, b: number, c: number) => void;
    readonly compute_list_slicer_styles: (a: number, b: number, c: number) => void;
    readonly compute_list_table_annotations: (a: number, b: number, c: number) => void;
    readonly compute_make_principal: (a: number, b: number, c: number, d: number) => void;
    readonly compute_map_slicer_disconnection_reason: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_map_slicer_invalidation_reason: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_merge_across: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_merge_and_center: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_merge_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_move_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_move_sheet: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_named_range_count: (a: number, b: number, c: number) => void;
    readonly compute_named_range_exists: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_parse_cell_ref: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_parse_date_input: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_parse_range_ref: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_patch_borders: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_patch_cell_properties_batch: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_patch_col_format: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_patch_col_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_patch_format_for_ranges: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_patch_row_format: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_patch_row_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_patch_workbook_settings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_pivot_compute_from_source: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_create: (a: number, b: number, c: number, d: number) => void;
    readonly compute_pivot_create_with_sheet: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_delete: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_pivot_get: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_pivot_get_all: (a: number, b: number, c: number, d: number) => void;
    readonly compute_pivot_get_all_items: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_get_imported_view_records: (a: number, b: number, c: number, d: number) => void;
    readonly compute_pivot_materialize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_materialize_mutation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_register_def: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_pivot_unregister_def: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_pivot_update: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_pivot_update_and_materialize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_prepare_date_value: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_prepare_time_value: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_preview_text_to_columns: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_protect_sheet: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_protect_sheet_with_options: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_protect_workbook: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_query_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_query_range_properties: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_query_ranges: (a: number, b: number, c: number, d: number) => void;
    readonly compute_reapply_filter: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_redo: (a: number, b: number, c: number) => void;
    readonly compute_regex_search: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_regex_search_all_sheets: (a: number, b: number, c: number, d: number) => void;
    readonly compute_register_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_relocate_cells: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_relocate_cells_yrs: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly compute_remove_binding: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_bindings_for_connection: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_calculated_column: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_cell_annotation_by_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_compute_sheet: (a: number, b: number, c: number, d: number) => void;
    readonly compute_remove_duplicates: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_remove_hf_image: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_horizontal_page_break: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_hyperlink: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_named_range: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_named_range_by_id: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_named_ranges_by_scope: (a: number, b: number, c: number, d: number) => void;
    readonly compute_remove_scenario: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_schema: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_remove_subtotals: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_remove_table_annotation: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_remove_table_column: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_table_data_row: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_remove_vertical_page_break: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_rename_compute_sheet: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_rename_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_rename_table_column: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_reorder_cf_rules: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_reorder_sheets: (a: number, b: number, c: number, d: number) => void;
    readonly compute_replace_all_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => void;
    readonly compute_reset_sheet_viewports: (a: number, b: number, c: number, d: number) => void;
    readonly compute_reset_viewport_state: (a: number, b: number, c: number, d: number) => void;
    readonly compute_reset_workbook_settings: (a: number, b: number, c: number) => void;
    readonly compute_resize_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_resize_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_resolve_cell_positions: (a: number, b: number, c: number, d: number) => void;
    readonly compute_resolve_named_range: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_resolve_table_format_at_cell: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_restore_scenario: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_rotate_floating_object_typed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_schema_infer_column: (a: number, b: number) => void;
    readonly compute_schema_infer_type: (a: number, b: number) => void;
    readonly compute_schema_resolve_editor: (a: number, b: number) => void;
    readonly compute_schema_validate: (a: number, b: number, c: number) => void;
    readonly compute_security_active: (a: number, b: number, c: number) => void;
    readonly compute_semantic_workbook_state_envelope: (a: number, b: number, c: number) => void;
    readonly compute_send_chart_backward: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_send_chart_to_back: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_send_floating_object_backward: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_send_floating_object_to_back: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_active_principal: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_active_scenario: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_array_formula: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_set_calculated_column_formula: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_calculation_mode: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_calculation_settings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_cell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_cell_annotation_by_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_cell_binary: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_cell_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_cell_properties_batch: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_cell_value_as_text: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_cell_value_parsed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_cell_values_parsed: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_cells_batch: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_col_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_col_format_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_col_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_col_width: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_col_width_chars: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_col_widths: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_col_widths_chars: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_column_filter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_column_schema: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_convergence_threshold: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_culture: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_current_time: (a: number, b: number) => void;
    readonly compute_set_custom_setting: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_date_value: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_set_default_pivot_table_style: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_default_slicer_style: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_default_table_style_id: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_document_properties: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_filter_sort_state: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_floating_object: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_floating_object_group: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_format_for_ranges: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_format_for_ranges_ui_state: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_frozen_panes: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_group_collapsed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_hf_image: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_hyperlink: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_iterative_calculation: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_iterative_calculation_enabled: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_level_collapsed: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_max_iterations: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_named_range: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_note_dimensions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_note_visible: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_outline_settings: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_print_area: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_print_settings: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_print_titles: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_range_schema: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_row_format: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_row_formats: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_row_height: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_schema_map: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_scroll_position: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_sheet_enable_calculation: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_sheet_hidden: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_sheet_protection_options: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_sheet_setting: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_sheet_visibility: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_slicer_selection: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_split_config: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_tab_color: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_set_table_annotation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_table_auto_calculated_columns: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_table_auto_expand: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_table_bool_option: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_table_style: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_table_totals_function: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_set_thread_resolved: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_time_value: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_set_use_precision_as_displayed: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_view_option: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_set_workbook_setting: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_set_workbook_settings: (a: number, b: number, c: number, d: number) => void;
    readonly compute_set_workbook_theme: (a: number, b: number, c: number, d: number) => void;
    readonly compute_settle_for_mirror: (a: number, b: number, c: number) => void;
    readonly compute_should_render_outlines: (a: number, b: number, c: number, d: number) => void;
    readonly compute_sign_check: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_sign_check_a1: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_solve: (a: number, b: number, c: number, d: number) => void;
    readonly compute_sort_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_stringify_cell_ref: (a: number, b: number, c: number, d: number) => void;
    readonly compute_stringify_range_ref: (a: number, b: number, c: number, d: number) => void;
    readonly compute_structure_change: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_sync_full_state: (a: number, b: number, c: number) => void;
    readonly compute_text_to_columns: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly compute_text_to_columns_simple: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => void;
    readonly compute_to_a1_display: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_to_a1_display_qualified: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_to_identity_formula: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_toggle_banded_cols: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_toggle_banded_rows: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_toggle_format_property: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_toggle_group_collapsed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_toggle_header_row: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_toggle_slicer_item: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_toggle_totals_row: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_undo: (a: number, b: number, c: number) => void;
    readonly compute_ungroup_columns: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_ungroup_rows: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_unhide_columns: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_unhide_rows: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_unlink_chart_from_table: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_unmerge_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_unprotect_sheet: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_unprotect_workbook: (a: number, b: number, c: number, d: number) => void;
    readonly compute_unregister_viewport: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_update_binding: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_calculated_column: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_update_cell_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_update_cf_ranges: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_cf_rule: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_chart: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_comment: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_update_comment_mentions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_update_custom_cell_style: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_update_custom_table_style: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_update_floating_object: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_floating_object_group: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_named_range: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_update_range_schema: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_refresh_metadata: (a: number, b: number, c: number, d: number, e: number, f: number, g: bigint, h: number) => void;
    readonly compute_update_rule_in_cf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_update_scenario: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_update_schema: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_update_shape_style: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_slicer_config: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_sparkline: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_update_viewport_bounds: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly compute_validate_and_clean_comments: (a: number, b: number, c: number, d: number) => void;
    readonly compute_validate_and_clean_merges: (a: number, b: number, c: number, d: number) => void;
    readonly compute_validate_cell_value: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_validate_formula_circular_reference: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly compute_validate_formula_syntax: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compute_validate_named_range_name: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly compute_wb_security_add_policy: (a: number, b: number, c: number, d: number) => void;
    readonly compute_wb_security_apply_template: (a: number, b: number, c: number, d: number) => void;
    readonly compute_wb_security_drain_events: (a: number, b: number, c: number) => void;
    readonly compute_wb_security_effective_access: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_wb_security_explain_access: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_wb_security_list_policies: (a: number, b: number, c: number) => void;
    readonly compute_wb_security_remove_policy: (a: number, b: number, c: number, d: number) => void;
    readonly compute_wb_security_remove_template: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly compute_wb_security_update_policy: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly pivot_compute: (a: number, b: number, c: number, d: number) => void;
    readonly pivot_compute_with_show_values_as: (a: number, b: number, c: number, d: number) => void;
    readonly pivot_detect_fields: (a: number, b: number) => void;
    readonly pivot_drill_down: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly pivot_validate_config: (a: number, b: number) => void;
    readonly table_add_column: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly table_adjust_structured_ref: (a: number, b: number, c: number) => void;
    readonly table_all_visible: (a: number, b: number) => void;
    readonly table_build_filter_dropdown: (a: number, b: number, c: number, d: number) => void;
    readonly table_build_slicer_cache: (a: number, b: number, c: number, d: number) => void;
    readonly table_cell_value_key: (a: number, b: number) => void;
    readonly table_cell_values_equal: (a: number, b: number, c: number) => void;
    readonly table_clear_slicer_selection: (a: number, b: number) => void;
    readonly table_compare_values: (a: number, b: number, c: number) => void;
    readonly table_compose_bitmaps: (a: number, b: number) => void;
    readonly table_compute_sort_order: (a: number, b: number, c: number, d: number) => void;
    readonly table_create_row_visibility: (a: number, b: number, c: number) => void;
    readonly table_create_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly table_evaluate_column_filter: (a: number, b: number, c: number) => void;
    readonly table_evaluate_top_bottom: (a: number, b: number, c: number) => void;
    readonly table_format_cell_display: (a: number, b: number) => void;
    readonly table_format_structured_ref: (a: number, b: number) => void;
    readonly table_generate_table_name: (a: number, b: number) => void;
    readonly table_get_built_in_styles: (a: number) => void;
    readonly table_get_column_at_grid_col: (a: number, b: number, c: number) => void;
    readonly table_get_column_by_id: (a: number, b: number, c: number, d: number) => void;
    readonly table_get_column_by_name: (a: number, b: number, c: number, d: number) => void;
    readonly table_get_column_data_range: (a: number, b: number, c: number, d: number) => void;
    readonly table_get_data_range: (a: number, b: number) => void;
    readonly table_get_header_range: (a: number, b: number) => void;
    readonly table_get_totals_formula: (a: number, b: number, c: number, d: number) => void;
    readonly table_get_totals_range: (a: number, b: number) => void;
    readonly table_is_in_data_range: (a: number, b: number, c: number, d: number) => void;
    readonly table_is_in_header_row: (a: number, b: number, c: number) => void;
    readonly table_is_in_table: (a: number, b: number, c: number, d: number) => void;
    readonly table_is_in_totals_row: (a: number, b: number, c: number) => void;
    readonly table_parse_structured_ref: (a: number, b: number, c: number) => void;
    readonly table_ranges_overlap: (a: number, b: number, c: number) => void;
    readonly table_remove_column: (a: number, b: number, c: number, d: number) => void;
    readonly table_rename_column: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly table_resize_table: (a: number, b: number, c: number) => void;
    readonly table_resolve_cell_format: (a: number, b: number, c: number, d: number) => void;
    readonly table_resolve_dynamic_filter: (a: number, b: number, c: number) => void;
    readonly table_resolve_structured_ref: (a: number, b: number, c: number, d: number) => void;
    readonly table_select_all_slicer_values: (a: number, b: number, c: number) => void;
    readonly table_select_slicer_values: (a: number, b: number, c: number) => void;
    readonly table_set_slicer_sort_order: (a: number, b: number, c: number) => void;
    readonly table_set_table_option: (a: number, b: number, c: number, d: number) => void;
    readonly table_set_totals_function: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly table_slicer_to_filter_criteria: (a: number, b: number) => void;
    readonly table_toggle_slicer_value: (a: number, b: number, c: number) => void;
    readonly table_toggle_totals_row: (a: number, b: number) => void;
    readonly table_validate_table_name: (a: number, b: number, c: number, d: number) => void;
    readonly table_value_in_list: (a: number, b: number, c: number) => void;
    readonly versioning_diff_semantic_workbook_states: (a: number, b: number, c: number) => void;
    readonly wasm_start: () => void;
    readonly xlsx_parse_lazy: (a: number, b: number, c: number) => void;
    readonly xlsx_parse_lazy_with_mode: (a: number, b: number, c: number, d: number) => void;
    readonly xlsx_version: (a: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
