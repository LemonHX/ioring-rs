use cmake::Config;

fn main() {
    //     use std::env;
    //     use std::path::PathBuf;
    //     const INCLUDE: &str = r#"
    // #include "ioringnt.h"
    // #include "libwinring.h"
    //     "#;
    //     let outdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");

    //     bindgen::Builder::default()
    //     .header_contents("include-file.h", INCLUDE)
    //     .clang_arg("-I./libwinring/include/")
    //     .clang_arg("-std=c++17")
    //     .clang_arg("-x")
    //     .clang_arg("c++")
    //     .opaque_type("std::.*")
    //     .blacklist_type("LPMONITORINFOEXA?W?")
    //     .blacklist_type("LPTOP_LEVEL_EXCEPTION_FILTER")
    //     .blacklist_type("MONITORINFOEXA?W?")
    //     .blacklist_type("PEXCEPTION_FILTER")
    //     .blacklist_type("PEXCEPTION_ROUTINE")
    //     .blacklist_type("PSLIST_HEADER")
    //     .blacklist_type("PTOP_LEVEL_EXCEPTION_FILTER")
    //     .blacklist_type("PVECTORED_EXCEPTION_HANDLER")
    //     .blacklist_type("_?L?P?CONTEXT")
    //     .blacklist_type("_?L?P?EXCEPTION_POINTERS")
    //     .blacklist_type("_?P?DISPATCHER_CONTEXT")
    //     .blacklist_type("_?P?EXCEPTION_REGISTRATION_RECORD")
    //     .blacklist_type("_?P?IMAGE_TLS_DIRECTORY.*")
    //     .blacklist_type("_?P?NT_TIB")
    //     .blacklist_type("tagMONITORINFOEXA")
    //     .blacklist_type("tagMONITORINFOEXW")
    //     .blacklist_function("AddVectoredContinueHandler")
    //     .blacklist_function("AddVectoredExceptionHandler")
    //     .blacklist_function("CopyContext")
    //     .blacklist_function("GetThreadContext")
    //     .blacklist_function("GetXStateFeaturesMask")
    //     .blacklist_function("InitializeContext")
    //     .blacklist_function("InitializeContext2")
    //     .blacklist_function("InitializeSListHead")
    //     .blacklist_function("InterlockedFlushSList")
    //     .blacklist_function("InterlockedPopEntrySList")
    //     .blacklist_function("InterlockedPushEntrySList")
    //     .blacklist_function("InterlockedPushListSListEx")
    //     .blacklist_function("LocateXStateFeature")
    //     .blacklist_function("QueryDepthSList")
    //     .blacklist_function("RaiseFailFastException")
    //     .blacklist_function("RtlCaptureContext")
    //     .blacklist_function("RtlCaptureContext2")
    //     .blacklist_function("RtlFirstEntrySList")
    //     .blacklist_function("RtlInitializeSListHead")
    //     .blacklist_function("RtlInterlockedFlushSList")
    //     .blacklist_function("RtlInterlockedPopEntrySList")
    //     .blacklist_function("RtlInterlockedPushEntrySList")
    //     .blacklist_function("RtlInterlockedPushListSListEx")
    //     .blacklist_function("RtlQueryDepthSList")
    //     .blacklist_function("RtlRestoreContext")
    //     .blacklist_function("RtlUnwindEx")
    //     .blacklist_function("RtlVirtualUnwind")
    //     .blacklist_function("SetThreadContext")
    //     .blacklist_function("SetUnhandledExceptionFilter")
    //     .blacklist_function("SetXStateFeaturesMask")
    //     .blacklist_function("UnhandledExceptionFilter")
    //     .blacklist_function("__C_specific_handler")
    //     .generate()
    //         .unwrap()
    //         .write_to_file(outdir.join("windows.rs"))
    //         .unwrap();
    let winring = Config::new("libwinring").build_target("ALL_BUILD").build();

    println!(
        "cargo:rustc-link-search=native={}/build/Debug",
        winring.display()
    );
    println!("cargo:rustc-link-lib=dylib=winring");

    println!("cargo:rustc-link-lib=dylib=ntdll");
}
