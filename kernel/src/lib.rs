#![no_std]

use assign_resources::assign_resources;
use embassy_rp::{
    Peri, bind_interrupts, gpio,
    peripherals::{self, DMA_CH0, SPI0},
    spi::{self, Spi},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};
use embassy_time::Delay;
use kernel_abi::KernelAbi;

use crate::display::ST7920;

mod display;

pub static KERNEL_ABI: OnceLock<KernelAbi> = OnceLock::new();
pub static DEVICE_DISPLAY: OnceLock<
    Mutex<CriticalSectionRawMutex, ST7920<Spi<SPI0, spi::Async>, gpio::Output>>,
> = OnceLock::new();

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
    let _ = DEVICE_DISPLAY.init(Mutex::new(display));

    // just for testing
    let mut d = DEVICE_DISPLAY.get().await.lock().await;
    d.set_pixel(0, 0, 1);
    d.flush(&mut delay).await;

    defmt::debug!("running kernel busy loop");
    loop {
        continue;
    }
}
