#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use embassy_executor::Executor;
use embassy_rp::multicore::Stack;
use kernel::{
    APP_EXIT, AssignedResources, DisplayResources, KERNEL_ABI, KERNEL_READY, kernel_entry,
    split_resources,
};
use kernel_abi::{ExitCode, KernelAbi};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut APP_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

#[unsafe(link_section = ".shell_flash_slot")]
static SHELL_FLASH: [u8; include_bytes!("../../target/thumbv6m-none-eabi/bin/shell.bin").len()] =
    *include_bytes!("../../target/thumbv6m-none-eabi/bin/shell.bin");

type AppEntry = extern "C" fn(*const KernelAbi) -> ExitCode;

fn get_shell_entry() -> AppEntry {
    let app_ptr = SHELL_FLASH.as_ptr() as usize | 1;
    unsafe { core::mem::transmute(app_ptr) }
}

fn core1_task() -> ! {
    defmt::debug!("core1 entry, waiting for kernel ready signal");
    while !KERNEL_READY.signaled() {
        cortex_m::asm::wfe();
    }

    // TODO this will go in a loop later
    APP_EXIT.reset();
    defmt::info!("spawn shell entry");
    let shell_entry = get_shell_entry();
    let exit_code = shell_entry(&KERNEL_ABI as *const KernelAbi);
    APP_EXIT.signal(exit_code);
    defmt::info!("got exit code '{}'", exit_code as u8);
    defmt::info!("core1 park");
    loop {
        cortex_m::asm::wfe();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::debug!("loader entry");
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);
    embassy_rp::multicore::spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(APP_STACK) },
        move || core1_task(),
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(core0_task(r).unwrap()))
}

#[embassy_executor::task]
async fn core0_task(r: AssignedResources) {
    defmt::debug!("kernel entry");
    kernel_entry(r).await;
}
