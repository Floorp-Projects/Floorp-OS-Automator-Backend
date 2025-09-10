#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Provide missing constant expected by upstream `libffi` crate API.
// Newer system libffi exposes FFI_TYPE_* constants but not ffi_type_enum_STRUCT symbol.
// We map it to the enum value used by libffi for struct types (per libffi headers FFI_TYPE_STRUCT = 13 / may vary but stable).
#[allow(non_upper_case_globals)]
pub const ffi_type_enum_STRUCT: u32 = FFI_TYPE_STRUCT as u32;
