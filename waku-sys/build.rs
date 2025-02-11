use std::env;
use std::env::set_current_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

extern crate cc;

fn submodules_init(project_dir: &Path) {
    let mark_file_path = ".submodules-initialized";

    // Check if the mark file exists
    if !Path::new(mark_file_path).exists() {
        // If mark file doesn't exist, initialize submodule
        if Command::new("git")
            .args(["submodule", "init"])
            .status()
            .expect("Failed to execute 'git submodule init'")
            .success()
            && Command::new("git")
                .args(["submodule", "update", "--recursive"])
                .status()
                .expect("Failed to execute 'git submodule update --recursive'")
                .success()
        {
            // Now, inside nwaku folder, run 'make update' to get nwaku's vendors
            let vendor_path = project_dir.join("vendor");
            set_current_dir(vendor_path).expect("Moving to vendor dir");

            if Command::new("make")
                .args(["update"])
                .status()
                .expect("Failed to execute 'make update'")
                .success()
            {
                std::fs::File::create(mark_file_path).expect("Failed to create mark file");
            } else {
                panic!("Failed to run 'make update' within nwaku folder.");
            }

            set_current_dir(project_dir).expect("Going back to project dir");

            println!("Git submodules initialized and updated successfully.");
        } else {
            panic!("Failed to initialize or update git submodules.");
        }
    } else {
        println!("Mark file '{mark_file_path}' exists. Skipping git submodule initialization.");
    }
}

fn build_nwaku_lib(project_dir: &Path) {
    let vendor_path = project_dir.join("vendor");
    set_current_dir(vendor_path).expect("Moving to vendor dir");

    let mut cmd = Command::new("make");
    cmd.arg("libwaku").arg("STATIC=1");
    cmd.status()
        .map_err(|e| println!("cargo:warning=make build failed due to: {e}"))
        .unwrap();

    set_current_dir(project_dir).expect("Going back to project dir");
}

fn generate_bindgen_code(project_dir: &Path) {
    let vendor_path = project_dir.join("vendor");
    let header_path = vendor_path.join("library/libwaku.h");

    cc::Build::new()
        .object(
            vendor_path
                .join("vendor/nim-libbacktrace/libbacktrace_wrapper.o")
                .display()
                .to_string(),
        )
        .compile("libbacktrace_wrapper");

    println!("cargo:rerun-if-changed={}", header_path.display());
    println!(
        "cargo:rustc-link-search={}",
        vendor_path.join("build").display()
    );
    println!("cargo:rustc-link-lib=static=waku");

    println!(
        "cargo:rustc-link-search={}",
        vendor_path
            .join("vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build")
            .display()
    );
    println!("cargo:rustc-link-lib=static=miniupnpc");

    println!(
        "cargo:rustc-link-search={}",
        vendor_path
            .join("vendor/nim-nat-traversal/vendor/libnatpmp-upstream")
            .display()
    );
    println!("cargo:rustc-link-lib=static=natpmp");

    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");

    println!(
        "cargo:rustc-link-search=native={}",
        vendor_path
            .join("vendor/nim-libbacktrace/install/usr/lib")
            .display()
    );
    println!("cargo:rustc-link-lib=static=backtrace");

    cc::Build::new()
        .file("src/cmd.c") // Compile the C file
        .compile("cmditems"); // Compile it as a library
    println!("cargo:rustc-link-lib=static=cmditems");

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

    submodules_init(&project_dir);
    build_nwaku_lib(&project_dir);
    generate_bindgen_code(&project_dir);
}
