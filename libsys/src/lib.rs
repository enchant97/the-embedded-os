#![no_std]
use kernel_abi::KernelAbi;

static mut ABI_PTR: *const KernelAbi = core::ptr::null();

/// Init the kernel abi, so libsys knows where to look.
///
/// Must be called once on program start.
#[unsafe(no_mangle)]
pub extern "C" fn init_abi(abi_ptr: &'static KernelAbi) {
    unsafe {
        ABI_PTR = abi_ptr;
    }
}

/// Get the current version of the kernel abi.
#[unsafe(no_mangle)]
pub extern "C" fn get_abi_version() -> u32 {
    unsafe { ((*ABI_PTR).get_version)() }
}
