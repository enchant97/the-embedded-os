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
}
