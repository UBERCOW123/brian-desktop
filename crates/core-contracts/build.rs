//! Injects `VENDOR_CORE_ROOT` pointing at `vendor/core` from the workspace root.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.join("../..");
    let vendor_core = workspace_root.join("vendor/core");
    println!(
        "cargo:rustc-env=VENDOR_CORE_ROOT={}",
        vendor_core.display()
    );
    println!("cargo:rerun-if-changed={}", vendor_core.display());
}
