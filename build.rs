use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=static=stdc++");
    let dir: PathBuf = ["third-party", "tree-sitter-lua", "src"].iter().collect();
    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .warnings(false)
        .file(dir.join("scanner.cc"))
        .warnings(false)
        .compile("tree-sitter-lua");
}
