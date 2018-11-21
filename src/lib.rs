#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_std]

extern crate embedded_hal;
extern crate nb;

// TODO: feature flag
extern crate fpa;

mod font;
mod hx1230;
mod pcd8544;

pub use pcd8544::gpio::Pcd8544Gpio;
pub use pcd8544::spi::Pcd8544Spi;
pub use pcd8544::Modes::*;

pub use hx1230::gpio::Hx1230Gpio;
pub use hx1230::Modes::*;

pub trait Display {
    /// x must be 0..83
    /// y must be 0..5
    fn set_position(&mut self, x: u8, y: u8);

    fn clear(&mut self);

    fn print_char(&mut self, c: u8);

    fn print(&mut self, s: &str);

    fn get_pixel_resolution() -> (u8, u8);
    fn get_char_resolution() -> (u8, u8);
}
