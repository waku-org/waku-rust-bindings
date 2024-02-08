use std::env;
use std::env::set_current_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build_nwaku_lib(project_dir: &Path) {
    let vendor_path = project_dir.join("vendor");

    set_current_dir(vendor_path).expect("Moving to vendor dir");

    let mut cmd = Command::new("make");
    cmd.arg("libwaku").arg("STATIC=true");
    cmd.status()
        .map_err(|e| println!("cargo:warning=make build failed due to: {e}"))
        .unwrap();

    set_current_dir(project_dir).expect("Going back to project dir");
}

fn generate_bindgen_code(project_dir: &Path) {
    let vendor_path = project_dir.join("vendor");
    let build_dir = vendor_path.join("build");
    let header_path = vendor_path.join("library/libwaku.h");

    println!("cargo:rustc-link-search={}", build_dir.display());
    println!("cargo:rustc-link-lib=static=waku");
    println!("cargo:rerun-if-changed={}", header_path.display());

    // Generate waku bindings with bindgen
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}", header_path.display()))
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

#[cfg(not(doc))]
fn main() {
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    build_nwaku_lib(&project_dir);
    generate_bindgen_code(&project_dir);
}
