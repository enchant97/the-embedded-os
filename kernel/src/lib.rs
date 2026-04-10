#![no_std]

use embassy_sync::once_lock::OnceLock;
use kernel_abi::KernelAbi;

pub static KERNEL_ABI: OnceLock<KernelAbi> = OnceLock::new();

pub fn kernel_entry() -> ! {
    loop {}
}
