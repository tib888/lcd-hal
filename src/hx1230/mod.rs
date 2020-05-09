//! The HX1230 is an inexpensive 96x68 monochrome LCD. It's a higher resolution replacement
//! for the PCD8544 (Nokia 5110) LCD. The problem is that this particular board is only exposes
//! a 3-wire SPI interface. The controller is capable of I2C, SPI 4 and 3-wire.
//! 3-wire means that the D/C signal is now the 9th bit added to each data unsigned char.
//! This prevents it from working with the hardware SPI interface, so it must be bit-banged.
//! The CE/CS line can be tied to ground to save a GPIO pin, but the RESET line must be toggled
//! upon power up to start using the display.

pub mod gpio;

pub trait Hx1230Base {
    type Error;
    fn command(&mut self, cmd: u8) -> Result<(), Self::Error>;
    fn data(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

pub enum Modes {
    Blank,
    Normal,
    Filled,
    Inverse,
}

pub trait Hx1230 {
    type Error;
    
    /// contrast < 32
    fn set_contrast(&mut self, contrast: u8) -> Result<(), Self::Error>;

    fn set_mode(&mut self, mode: Modes) -> Result<(), Self::Error>;

    fn flip_horizontal(&mut self, flip: bool) -> Result<(), Self::Error>;
    fn flip_vertical(&mut self, flip: bool) -> Result<(), Self::Error>;

    fn init(&mut self) -> Result<(), Self::Error>;

    // TODO
    // 8 lines of 8 pixels and 1 line of 4 pixels
    // fn draw_buffer(&mut self, buffer: &[u8; 9 * 96])
}
