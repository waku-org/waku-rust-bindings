use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_go_bin() -> String {
    if cfg!(target_family = "unix") {
        let output = String::from_utf8(
            Command::new("/usr/bin/which")
                .arg("go")
                .output()
                .map_err(|e| println!("cargo:warning=Couldn't find `which` command: {}", e))
                .expect("`which` command not found")
                .stdout,
        )
        .expect("which output couldnt be parsed");

        if output.is_empty() {
            println!("cargo:warning=Couldn't find go binary installed, please ensure that it is installed and/or withing the system paths");
            panic!("Couldn't find `go` binary installed");
        }
        output.trim().to_string()
    } else if cfg!(target_family = "windows") {
        "go".into()
    } else {
        panic!("OS not supported!");
    }
}

fn build_go_waku_lib(go_bin: &str, project_dir: &Path) {
    // Build go-waku static lib
    // build command taken from waku make file:
    // https://github.com/status-im/go-waku/blob/eafbc4c01f94f3096c3201fb1e44f17f907b3068/Makefile#L115
    let output_lib = project_dir.join("vendor/build/lib/libgowaku.a");
    let library_path = project_dir.join("vendor/library");
    Command::new(go_bin)
        .env("CGO_ENABLED", "1")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(output_lib.display().to_string())
        .arg(library_path.display().to_string())
        .status()
        .map_err(|e| println!("cargo:warning=go build failed due to: {}", e))
        .unwrap();
}

fn generate_bindgen_code(project_dir: &Path) {
    let lib_dir = project_dir.join("vendor/build/lib");

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

fn main() {
    let go_bin = get_go_bin();

    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    build_go_waku_lib(&go_bin, &project_dir);
    generate_bindgen_code(&project_dir);
}
