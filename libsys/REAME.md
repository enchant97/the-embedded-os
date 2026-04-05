# Libsys
A system library wrapper around the Kernel ABI definitions.

## Usage

```rs
// file: src/main.rs
#![no_std]
#![no_main]

#[libsys::main_entrypoint]
fn main() -> libsys::ExitCode {
    let _ = libsys::core::get_abi_version();
    libsys::ExitCode::Ok
}
```
