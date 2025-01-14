use std::env;
use std::path::{Path, PathBuf};

extern crate cc;

fn generate_bindgen_code(project_dir: &Path) {
    let libwaku_path = project_dir.join("libwaku/"); // dir with libs, objects and header

    println!("cargo:rerun-if-changed={}", libwaku_path.display());
    println!(
        "cargo:rustc-link-search={}",
        libwaku_path.join("build").display()
    );
    println!("cargo:rustc-link-lib=static=waku");

    println!("cargo:rustc-link-search={}", libwaku_path.display());
    println!("cargo:rustc-link-lib=static=miniupnpc");

    println!("cargo:rustc-link-search={}", libwaku_path.display());

    println!("cargo:rustc-link-lib=static=natpmp");

    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");

    println!("cargo:rustc-link-search=native={}", libwaku_path.display());
    println!("cargo:rustc-link-lib=static=backtrace");

    cc::Build::new()
        .file("src/cmd.c") // Compile the C file
        .compile("cmditems"); // Compile it as a library
    println!("cargo:rustc-link-lib=static=cmditems");

    // Generate waku bindings with bindgen
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}", libwaku_path.join("libwaku.h").display()))
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
    // Notice that CARGO_MANIFEST_DIR is different depending on "cargo build" or "cargo package"
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    generate_bindgen_code(&project_dir);
}
