#![no_std]

#[repr(C)]
pub struct KernelAbi {
    pub get_version: extern "C" fn() -> u32,
}
