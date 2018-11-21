#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_std]

extern crate embedded_hal;
extern crate nb;

// TODO: feature flag
extern crate fpa;

pub mod font;
pub mod hx1230;
pub mod pcd8544;

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
