//! ST7920 SPI Driver
//!
//! Based on: <https://github.com/wjakobczyk/st7920/tree/master>
//! Based on: <https://github.com/enchant97/micropython-st7920>

use embedded_graphics::{
    Pixel,
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, OriginDimensions, Size},
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::{delay::DelayNs, spi::SpiBus};

#[repr(u8)]
enum Instruction {
    BasicFunction = 0x30,
    ExtendedFunction = 0x34,
    ClearScreen = 0x01,
    EntryMode = 0x06,
    DisplayOnCursorOff = 0x0C,
    GraphicsOn = 0x36,
    SetGraphicsAddress = 0x80,
}

const INIT_INSTRUCTIONS: [Instruction; 7] = [
    Instruction::BasicFunction,
    Instruction::BasicFunction,
    Instruction::DisplayOnCursorOff,
    Instruction::ClearScreen,
    Instruction::EntryMode,
    Instruction::ExtendedFunction,
    Instruction::GraphicsOn,
];
const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const ROW_SIZE: usize = (WIDTH / 8) as usize;
const BUFFER_SIZE: usize = ROW_SIZE * HEIGHT as usize;

pub struct ST7920<SPI, CS> {
    spi: SPI,
    cs: CS,
    buffer: [u8; BUFFER_SIZE],
    flip: bool,
}

impl<SPI, CS> ST7920<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub const fn width() -> u32 {
        WIDTH
    }

    pub const fn height() -> u32 {
        HEIGHT
    }

    pub fn new(spi: SPI, cs: CS, flip: bool) -> Self {
        let buffer = [0; BUFFER_SIZE];
        Self {
            spi,
            cs,
            buffer,
            flip,
        }
    }

    async fn write_command(&mut self, byte: u8, delay: &mut impl DelayNs) {
        self.spi.write(&[0xf8]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[byte & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[(byte << 4) & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
    }

    async fn write_data(&mut self, byte: u8, delay: &mut impl DelayNs) {
        self.spi.write(&[0xf8 | 0x02]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[byte & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[(byte << 4) & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
    }

    async fn set_graphics_address(&mut self, x: u8, y: u8, delay: &mut impl DelayNs) {
        self.write_command(Instruction::SetGraphicsAddress as u8 | y, delay)
            .await;
        self.write_command(Instruction::SetGraphicsAddress as u8 | x, delay)
            .await;
    }

    pub async fn init(&mut self, delay: &mut impl DelayNs) {
        self.cs.set_high().unwrap();
        for instruction in INIT_INSTRUCTIONS {
            self.write_command(instruction as u8, delay).await;
            delay.delay_ms(2).await;
        }
        self.cs.set_low().unwrap();
    }

    #[inline]
    fn set_pixel_unchecked(&mut self, mut x: u8, mut y: u8, mut val: u8) {
        if val > 1 {
            val = 0;
        }
        if self.flip {
            y = (Self::height() - 1) as u8 - y;
            x = (Self::width() - 1) as u8 - x;
        }
        let idx = y as usize * ROW_SIZE + x as usize / 8;
        let x_mask = 0x80 >> (x % 8);
        if val != 0 {
            self.buffer[idx] |= x_mask;
        } else {
            self.buffer[idx] &= !x_mask;
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u8, y: u8, val: u8) {
        if x < Self::width() as u8 && y < Self::height() as u8 {
            self.set_pixel_unchecked(x, y, val);
        }
    }

    #[inline]
    pub async fn flush(&mut self, delay: &mut impl DelayNs) {
        self.cs.set_high().unwrap();
        for y in 0..Self::height() as u8 {
            let row_offset = (y as u32) * 16;
            if y < 32 {
                self.set_graphics_address(0, y, delay).await;
            } else {
                self.set_graphics_address(8, y - 32, delay).await;
            }
            for i in 0..16 {
                self.write_data(self.buffer[(row_offset + i) as usize], delay)
                    .await;
            }
        }
        self.cs.set_low().unwrap();
    }
}

impl<SPI, CS> OriginDimensions for ST7920<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(Self::width(), Self::height())
    }
}

impl<SPI, CS> DrawTarget for ST7920<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    type Color = BinaryColor;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            self.set_pixel(
                coord.x as u8,
                coord.y as u8,
                match color {
                    BinaryColor::On => 1,
                    BinaryColor::Off => 0,
                },
            );
        }
        Ok(())
    }
}
