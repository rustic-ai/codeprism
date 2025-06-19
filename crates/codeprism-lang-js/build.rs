fn main() {
    // The tree-sitter-javascript and tree-sitter-typescript crates
    // already compile their grammars, so we don't need to do it here.
    // This build.rs is kept for future custom grammar compilation if needed.

    println!("cargo:rerun-if-changed=build.rs");
}
