#![no_std]
#![allow(static_mut_refs)]

use core::{ffi::c_void, str};

use assign_resources::assign_resources;
use embassy_rp::{
    Peri, bind_interrupts, gpio,
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

use crate::display::ST7920;

mod display;

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

    defmt::debug!("kernel ready");
    KERNEL_READY.signal(());

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
