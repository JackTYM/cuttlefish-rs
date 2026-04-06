//! Build script for cuttlefish-api.
//!
//! Sets default WEBUI_DIR environment variable for rust-embed if not specified.

fn main() {
    // If WEBUI_DIR is not set, use the placeholder directory
    // This allows development builds to compile without the WebUI
    if std::env::var("WEBUI_DIR").is_err() {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
        let placeholder = format!("{}/webui-placeholder", manifest_dir);
        println!("cargo::rustc-env=WEBUI_DIR={}", placeholder);
    }

    // Re-run if WEBUI_DIR changes
    println!("cargo::rerun-if-env-changed=WEBUI_DIR");
}
