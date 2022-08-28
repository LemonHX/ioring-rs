use cmake::Config;

fn main() {
    let winring = Config::new("libwinring").build_target("ALL_BUILD"). build();

    println!("cargo:rustc-link-search=native={}/build/Debug", winring.display());
    println!("cargo:rustc-link-lib=dylib=winring");

    println!("cargo:rustc-link-lib=dylib=ntdll");
}
