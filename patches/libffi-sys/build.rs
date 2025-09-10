use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Prefer pkg-config to discover system libffi
    let mut include_args: Vec<String> = Vec::new();
    if let Ok(lib) = pkg_config::Config::new().print_system_libs(false).probe("libffi") {
        for p in lib.include_paths { include_args.push(format!("-I{}", p.display())); }
        println!("cargo:rustc-link-lib=ffi");
    } else {
        println!("cargo:warning=Falling back to default libffi include paths");
        println!("cargo:rustc-link-lib=ffi");
        include_args.push("-I/usr/include".into());
        include_args.push("-I/usr/include/x86_64-linux-gnu".into());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wrapper = out_dir.join("wrapper.h");
    fs::write(&wrapper, "#include <ffi.h>\n").unwrap();

    let mut builder = bindgen::Builder::default()
        .header(wrapper.to_string_lossy())
        .allowlist_type("ffi_.*")
        .allowlist_function("ffi_.*")
        .allowlist_var("ffi_type_.*")
        .allowlist_var("FFI_.*")
        .derive_default(true)
        .derive_debug(true)
        .layout_tests(false);

    for arg in &include_args { builder = builder.clang_arg(arg); }

    let bindings = builder.generate().expect("Unable to generate libffi bindings");
    let out_bindings = out_dir.join("bindings.rs");
    bindings.write_to_file(&out_bindings).expect("Couldn't write bindings!");
}
