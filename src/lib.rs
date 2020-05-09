#![deny(unsafe_code)]
#![no_std]

// TODO: feature flags?
pub mod font;
pub mod hx1230;
pub mod pcd8544;

pub trait Display {
    type Error;

    /// x must be 0..83
    /// y must be 0..5
    fn set_position(&mut self, x: u8, y: u8) -> Result<(), Self::Error>;

    fn clear(&mut self) -> Result<(), Self::Error>;

    fn print_char(&mut self, c: u8) -> Result<(), Self::Error>;

    fn print(&mut self, s: &[u8]) -> Result<(), Self::Error> {
        for c in s {
            self.print_char(*c)?;
        }
        Ok(())
    }

    /// returns (cols, rows)
    fn get_pixel_resolution(&self) -> (u8, u8);

    /// returns (cols, rows)
    fn get_char_resolution(&self) -> (u8, u8);
}
