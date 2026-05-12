//! Commonly used misc functions and types
use kernel_abi::{ExitCode, KernelAbi};

/// A runnable user app entry function.
///
/// Either points to a location in user apps flash or user memory.
pub type AppEntry = extern "C" fn(*const KernelAbi) -> ExitCode;
