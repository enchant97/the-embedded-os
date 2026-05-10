#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use assign_resources::assign_resources;
use core::ptr::addr_of_mut;
use core::{ffi::c_void, str};
use embassy_executor::Executor;
use embassy_rp::{
    Peri, bind_interrupts, gpio,
    multicore::Stack,
    peripherals::{self, DMA_CH0},
    spi::{self, Spi},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Delay;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, ascii::FONT_4X6},
    pixelcolor::BinaryColor,
    prelude::{Point, Size},
    text::{Alignment, Text, renderer::TextRenderer},
};
use kernel_abi::{ExitCode, FileDescriptor, KernelAbi};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::display::ST7920;

mod display;

unsafe extern "C" {
    static __shell_flash_start: u32;
    static __shell_flash_end: u32;
}

#[allow(unused)]
#[unsafe(link_section = ".shell_flash_slot")]
static SHELL_FLASH: [u8; include_bytes!("../../target/thumbv6m-none-eabi/bin/shell.bin").len()] =
    *include_bytes!("../../target/thumbv6m-none-eabi/bin/shell.bin");

static mut APP_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

type AppEntry = extern "C" fn(*const KernelAbi) -> ExitCode;

pub static KERNEL_ABI: KernelAbi = KernelAbi {
    get_version: abi_get_version,
    exit: abi_exit,
    malloc: abi_malloc,
    free: abi_free,
    write: abi_write,
    flush: abi_flush,
    seek: abi_seek,
    mmap: abi_mmap,
    ioctl: abi_ioctl,
};
pub static KERNEL_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static APP_EXIT: Signal<CriticalSectionRawMutex, ExitCode> = Signal::new();

static FLUSH_DISPLAY_SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();
// NOTE "static mut" is generally not a good idea. But may be required for this kernel?
// display buffer that either stores text or pixel data, depending on current mode.
static mut DISPLAY_FB: [u8; 128 * 64] = [0; 128 * 64];

assign_resources! {
    display: DisplayResources {
        spi: SPI0,
        cs: PIN_17,
        sck: PIN_18,
        mosi: PIN_19,
        dma: DMA_CH0,
    }
}

bind_interrupts!(struct Irqs{
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

extern "C" fn abi_get_version() -> u32 {
    1
}

extern "C" fn abi_exit(code: ExitCode) -> ! {
    todo!()
}

extern "C" fn abi_malloc(size: usize) -> *mut u8 {
    todo!()
}

extern "C" fn abi_free(ptr: *mut u8) {
    todo!()
}

extern "C" fn abi_write(fd: FileDescriptor, buff: *const u8, buff_len: usize) {
    todo!()
}

extern "C" fn abi_flush(fd: FileDescriptor) {
    match fd {
        FileDescriptor::Display => FLUSH_DISPLAY_SIG.signal(()),
    }
}

extern "C" fn abi_seek(fd: FileDescriptor, offset: usize) {
    todo!()
}

extern "C" fn abi_mmap(fd: FileDescriptor) -> *mut c_void {
    match fd {
        FileDescriptor::Display => unsafe { DISPLAY_FB.as_mut_ptr() as *mut c_void },
    }
}

extern "C" fn abi_ioctl(
    fd: FileDescriptor,
    op: usize,
    in_arg: *const c_void,
    out_arg: *mut c_void,
) -> ExitCode {
    ExitCode::Ok
}

pub async fn kernel_entry(r: AssignedResources) -> ! {
    let mut display_spi_config = spi::Config::default();
    display_spi_config.polarity = spi::Polarity::IdleLow;
    display_spi_config.phase = spi::Phase::CaptureOnFirstTransition;
    display_spi_config.frequency = 600000;
    let display_spi = Spi::new_txonly(
        r.display.spi,
        r.display.sck,
        r.display.mosi,
        r.display.dma,
        Irqs,
        display_spi_config,
    );
    let display_spi_cs = gpio::Output::new(r.display.cs, gpio::Level::Low);
    let mut display = ST7920::new(display_spi, display_spi_cs, false);
    let mut delay = Delay {};
    display.init(&mut delay).await;
    display.flush(&mut delay).await;

    KERNEL_READY.signal(());
    defmt::debug!("waking core1");
    cortex_m::asm::sev();

    let font = FONT_4X6;
    let text_style = MonoTextStyle::new(&font, BinaryColor::On);

    loop {
        defmt::debug!("waiting for next display flush");
        FLUSH_DISPLAY_SIG.wait().await;
        // assumes always in text-mode
        let mut point = Point::new(0, font.character_size.height as i32);
        let n_lines = 64 / font.character_size.height as usize;
        let line_length = 128 / font.character_size.width as usize;
        for line_i in 0..n_lines {
            let line;
            unsafe {
                line = &DISPLAY_FB[line_i * line_length..(line_i + 1) * line_length];
            }
            defmt::debug!("{:?}", line);
            Text::with_alignment(
                str::from_utf8(line).unwrap().trim_end_matches("\0"),
                point,
                text_style,
                Alignment::Left,
            )
            .draw(&mut display)
            .unwrap();
            point += Size::new(0, text_style.line_height());
        }
        display.flush(&mut delay).await;
        defmt::debug!("done display flush");
    }
}

fn get_shell_entry() -> AppEntry {
    unsafe {
        // `| 1` enables Thumb mode
        let addr = &raw const __shell_flash_start as usize | 1;
        core::mem::transmute(addr)
    }
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
        // TODO replace addr_of_mut with newer implementation
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
