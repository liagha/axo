use std::{env, path::PathBuf};

const LLVM_SYS_VERSION: &str = "191";

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let llvm_target_dir = match target.as_str() {
        "x86_64-unknown-linux-gnu" => "x86_64-linux",
        "aarch64-unknown-linux-gnu" => "aarch64-linux",
        "x86_64-apple-darwin" => "x86_64-macos",
        "aarch64-apple-darwin" => "aarch64-macos",
        _ => {
            panic!(
                "unsupported target `{}` for bundled llvm/. supported targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin",
                target
            );
        }
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let llvm_dist = manifest_dir.join("llvm").join(llvm_target_dir);

    if !llvm_dist.exists() {
        panic!(
            "llvm bundle not found for target {target}. Expected {}",
            llvm_dist.display()
        );
    }

    let lib_dir = llvm_dist.join("lib");
    println!("library directory: {}", lib_dir.display());
    let include_dir = llvm_dist.join("include");
    println!("include directory: {}", include_dir.display());

    assert!(lib_dir.exists(), "{} not found", lib_dir.display());
    assert!(include_dir.exists(), "{} not found", include_dir.display());

    let prefix_var = format!("LLVM_SYS_{}_PREFIX", LLVM_SYS_VERSION);
    println!("cargo:rustc-env={}={}", prefix_var, llvm_dist.display());
    println!("cargo:rustc-env=LIBRARY_PATH={}", lib_dir.display());

    println!("cargo:rustc-env=AXO_LLVM_PREFIX={}", llvm_dist.display());

    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-changed=llvm");
}
