//! Pure and service method emission.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use super::ir::{WasmDescriptor, WasmMethod};
use super::names::to_snake_case;
use super::params::build_params_and_conversions;
use super::returns::build_return_handling;
use super::types::is_direct_return;

/// Keep methods hidden from the generated TypeScript bridge while making their
/// direct WASM transport name an explicit part of the binding contract.
///
/// `skip(ts_bridge)` is intentionally not `skip(wasm)`: handwritten system
/// mutations use the generated wasm-bindgen function directly. An explicit
/// `js_name` prevents a future Rust/wasm-bindgen naming change from making the
/// transport lookup (`wasm[command]`) diverge from the command name.
fn wasm_bindgen_attr(method: &WasmMethod, fn_name: &Ident) -> TokenStream {
    if is_ts_bridge_skipped(method) {
        let export_name = fn_name.to_string();
        quote! { #[wasm_bindgen(js_name = #export_name)] }
    } else {
        quote! { #[wasm_bindgen] }
    }
}

/// Give ts-bridge-only functions a private Rust/WASM symbol while preserving
/// the public JavaScript command name through `js_name`. This makes the
/// handwritten transport export a distinct binding rather than relying on
/// wasm-bindgen's implicit Rust-name mapping.
fn wasm_rust_fn_name(method: &WasmMethod, fn_name: &Ident) -> Ident {
    if is_ts_bridge_skipped(method) {
        format_ident!("__{}", fn_name)
    } else {
        fn_name.clone()
    }
}

fn is_ts_bridge_skipped(method: &WasmMethod) -> bool {
    method.skip_targets.iter().any(|target| target == "ts_bridge")
}

pub(super) fn emit_pure_method(
    _desc: &WasmDescriptor,
    method: &WasmMethod,
    type_snake: &str,
    type_ident: &Ident,
) -> TokenStream {
    let fn_name = if type_snake.is_empty() {
        format_ident!("{}", method.name)
    } else {
        format_ident!("{}_{}", type_snake, method.name)
    };
    let method_ident = format_ident!("{}", method.name);

    let (wasm_params, conversion_stmts, call_args) = build_params_and_conversions(&method.params);

    let (return_type_tokens, result_conversion) =
        build_return_handling(&method.return_type, method.is_fallible);
    let wasm_bindgen_attr = wasm_bindgen_attr(method, &fn_name);
    let wasm_fn_name = wasm_rust_fn_name(method, &fn_name);

    let call_expr = if method.is_fallible {
        quote! {
            let result = #type_ident::#method_ident(#(#call_args),*)
                .map_err(|e| wasm_bindgen::JsError::new(&bridge_types::bridge_format_err!(e)))?;
        }
    } else {
        quote! {
            let result = #type_ident::#method_ident(#(#call_args),*);
        }
    };

    quote! {
        #wasm_bindgen_attr
        pub fn #wasm_fn_name(#(#wasm_params),*) -> #return_type_tokens {
            #(#conversion_stmts)*
            #call_expr
            #result_conversion
        }
    }
}
/// Emit a service method (read or write).
pub(super) fn emit_service_method(
    desc: &WasmDescriptor,
    method: &WasmMethod,
    type_snake: &str,
    _type_ident: &Ident,
    is_write: bool,
) -> TokenStream {
    let fn_name = if type_snake.is_empty() {
        format_ident!("{}", method.name)
    } else {
        format_ident!("{}_{}", type_snake, method.name)
    };

    // Get the key param name from the service meta
    let key_param_name = desc
        .service
        .as_ref()
        .map(|s| s.key_param.clone())
        .unwrap_or_else(|| "id".to_string());
    let key_param = format_ident!("{}", key_param_name);

    let (wasm_params, conversion_stmts, call_args) = build_params_and_conversions(&method.params);
    let wasm_bindgen_attr = wasm_bindgen_attr(method, &fn_name);
    let wasm_fn_name = wasm_rust_fn_name(method, &fn_name);

    // Key param comes first
    let mut all_wasm_params = vec![quote! { #key_param: &str }];
    all_wasm_params.extend(wasm_params);

    let (return_type_tokens, _result_conversion) = build_return_handling(
        &method.return_type,
        method.is_fallible || desc.service.is_some(),
    );

    let internal_snake = to_snake_case(&desc.type_name);
    let helper_fn = if is_write {
        format_ident!("__with_write_{}", internal_snake)
    } else {
        format_ident!("__with_read_{}", internal_snake)
    };

    let method_ident = format_ident!("{}", method.name);

    let inner_call = if is_write {
        if method.is_fallible {
            quote! {
                instance.#method_ident(#(#call_args),*)
                    .map_err(|e| wasm_bindgen::JsError::new(&bridge_types::bridge_format_err!(e)))?
            }
        } else {
            quote! {
                instance.#method_ident(#(#call_args),*)
            }
        }
    } else if method.is_fallible {
        quote! {
            instance.#method_ident(#(#call_args),*)
                .map_err(|e| wasm_bindgen::JsError::new(&bridge_types::bridge_format_err!(e)))?
        }
    } else {
        quote! {
            instance.#method_ident(#(#call_args),*)
        }
    };

    // For serde returns, we need to do the conversion inside the closure
    let needs_serde_return = method
        .return_type
        .as_ref()
        .map(|r| !is_direct_return(r))
        .unwrap_or(false);

    // For bytes-tuple returns, we need to destructure and convert inside the closure
    let needs_bytes_tuple_return = method
        .return_type
        .as_ref()
        .map(|r| r.is_bytes_tuple)
        .unwrap_or(false);

    let closure_body = if needs_serde_return {
        quote! {
            let result = #inner_call;
            serde::Serialize::serialize(
                &result,
                &serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true),
            )
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
        }
    } else if needs_bytes_tuple_return {
        quote! {
            let result = #inner_call;
            let (bytes, metadata) = result;
            let js_bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let js_metadata = serde::Serialize::serialize(
                &metadata,
                &serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true),
            )
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))?;
            let arr = js_sys::Array::new_with_length(2);
            arr.set(0, js_bytes.into());
            arr.set(1, js_metadata);
            Ok(arr.into())
        }
    } else {
        let has_return = method.return_type.is_some();
        if has_return {
            quote! {
                let result = #inner_call;
                Ok(result)
            }
        } else {
            quote! {
                #inner_call;
                Ok(())
            }
        }
    };

    quote! {
        #wasm_bindgen_attr
        pub fn #wasm_fn_name(#(#all_wasm_params),*) -> #return_type_tokens {
            #(#conversion_stmts)*
            #helper_fn(#key_param, |instance| {
                #closure_body
            })
        }
    }
}
