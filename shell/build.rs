use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // ensure it's on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("linker.ld"))
        .unwrap()
        .write_all(include_bytes!("linker.ld"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    //ensure the build script is only re-run when linker has changed
    println!("cargo:rerun-if-changed=linker.ld");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlinker.ld");
}
