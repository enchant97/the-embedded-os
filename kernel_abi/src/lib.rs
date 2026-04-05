#![no_std]

/// Used to report the exit code of the program.
#[repr(C)]
pub enum ExitCode {
    /// Success
    Ok = 0,
    /// Generic error, use a specific one if available
    GeneralError = 1,
}

#[repr(C)]
pub struct KernelAbi {
    pub get_version: extern "C" fn() -> u32,
    pub exit: extern "C" fn(ExitCode) -> !,
    /// Memory allocation request
    pub malloc: extern "C" fn(size: usize) -> *mut u8,
    /// Memory removal request
    pub free: extern "C" fn(ptr: *mut u8),
}
