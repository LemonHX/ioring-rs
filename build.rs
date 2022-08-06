fn main() {
    use std::env;
    use std::path::PathBuf;
    const INCLUDE: &str = r#"
#include "ioringnt.h"
#include "libwinring.h"
    "#;
    let outdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");

    bindgen::Builder::default()
    .header_contents("include-file.h", INCLUDE)
    .clang_arg("-I./src/libwinring/include/")
    .clang_arg("-std=c++17")
    .clang_arg("-x")
    .clang_arg("c++")
    .opaque_type("std::.*")
    .generate()
        .unwrap()
        .write_to_file(outdir.join("windows.rs"))
        .unwrap();
}