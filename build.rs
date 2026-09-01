// Compile the Elm visibility-toggle app to JS at build time. The output is
// embedded into the page by src/lib.rs via include_str!(OUT_DIR/elm.js), so the
// generated HTML stays self-contained. Requires `elm` on PATH.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=elm/src/Main.elm");
    println!("cargo:rerun-if-changed=elm/elm.json");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let output = format!("{out_dir}/elm.js");

    let status = Command::new("elm")
        .current_dir("elm")
        .args(["make", "src/Main.elm", "--optimize", &format!("--output={output}")])
        .status()
        .expect("failed to run `elm make` (is elm installed and on PATH?)");

    assert!(status.success(), "`elm make` failed");
}
