use std::env;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Build go-waku static lib
    // build command taken from waku make file:
    // https://github.com/status-im/go-waku/blob/eafbc4c01f94f3096c3201fb1e44f17f907b3068/Makefile#L115
    let output_lib = "libgowaku.a";
    set_current_dir("./vendor").unwrap();
    Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("./build/lib/{output_lib}"))
        .arg("./library/")
        .status()
        .map_err(|e| println!("cargo:warning={}", e))
        .unwrap();
    set_current_dir("../").unwrap();
    let mut lib_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    lib_dir.push("vendor");
    lib_dir.push("build");
    lib_dir.push("lib");

    println!("cargo:rustc-link-search={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=gowaku");
    println!("cargo:rerun-if-changed=libgowaku.h");

    // Generate waku bindings with bindgen
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/{}", lib_dir.display(), "libgowaku.h"))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
