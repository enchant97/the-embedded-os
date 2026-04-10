#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use embassy_executor::Executor;
use embassy_rp::multicore::Stack;
use static_cell::StaticCell;

use kernel::{KERNEL_ABI, kernel_entry};
use kernel_abi::KernelAbi;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    embassy_rp::multicore::spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.spawn(core1_task().unwrap()));
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(core0_task().unwrap()))
}

#[embassy_executor::task]
async fn core0_task() {
    kernel_entry();
}

#[embassy_executor::task]
async fn core1_task() {
    // NOTE maybe make this a lib so we don't have to include_bytes
    let shell = include_bytes!("../../target/thumbv6m-none-eabi/bin/shell.bin");
    let shell_entry: extern "C" fn(&KernelAbi) -> u8 =
        unsafe { core::mem::transmute(shell.as_ptr()) };
    let abi = KERNEL_ABI.get().await;
    shell_entry(abi);
}
