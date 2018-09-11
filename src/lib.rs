#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_std]

extern crate nb;

extern crate embedded_hal;

// TODO: feature flag
extern crate fpa;

pub mod demo;
mod font;
mod pcd8544_base;
mod pcd8544_gpio;
mod pcd8544_spi;

pub use pcd8544_gpio::Pcd8544Gpio;
pub use pcd8544_spi::Pcd8544Spi;

use pcd8544_base::Pcd8544Base; //private for this module

pub enum Modes {
    Blank = 0b0001000,
    Normal = 0b0001100,
    Filled = 0b0001001,
    Inverse = 0b0001101,
}

pub trait Pcd8544 {
    /// voltage_coefficient < 90 => VLCD = 3.06 + voltage_coefficient * 0.06; VLCD must be less than 8.5V
    /// temp_coefficient < 4
    /// bias < 8 => Vbias = 1/(bias + 4) * VLCD
    fn set_lcd_coefficients(&mut self, voltage_coefficient: u8, temp_coefficient: u8, bias: u8);

    fn set_mode(&mut self, mode: Modes);

    /// x must be 0..83
    /// y must be 0..5
    fn set_position(&mut self, x: u8, y: u8);

    fn clear(&mut self);

    fn init(&mut self);

    fn print_char(&mut self, c: u8);

    fn print(&mut self, s: &str);

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; 6 * 84]);
}

impl<T: Pcd8544Base> Pcd8544 for T {
    // fn set_function_set(
    //     &mut self,
    //     power_down: bool,
    //     vertical_addressing: bool,
    //     use_extended_instruction_set: bool,
    // ) {
    //     self.command(
    //         0b0010_0000u8
    //             | if power_down { 0b100u8 } else { 0u8 }
    //             | if vertical_addressing { 0b010u8 } else { 0u8 }
    //             | if use_extended_instruction_set {
    //                 0b001u8
    //             } else {
    //                 0u8
    //             },
    //     );
    // }

    /// voltage_coefficient < 90 => VLCD = 3.06 + voltage_coefficient * 0.06; VLCD must be less than 8.5V
    /// temp_coefficient < 4
    /// bias < 8 => Vbias = 1/(bias + 4) * VLCD
    fn set_lcd_coefficients(&mut self, voltage_coefficient: u8, temp_coefficient: u8, bias: u8) {
        assert!(voltage_coefficient < 91);
        assert!(temp_coefficient < 4);
        assert!(bias < 8);
        self.command(0b0010_0001); // use_extended_instruction_set = true
        self.command(0b1000_0000 | (voltage_coefficient & 0b0111_1111)); // try 0x31 (for 3.3V red SparkFun), 0x38 (for 3.3V blue SparkFun), 0x3F if your display is too dark, or 0 to 90 if experimenting
        self.command(0b0000_0100 | (temp_coefficient & 0b0000_0011)); // set temp coefficient
        self.command(0b0001_0000 | (bias & 0b0000_0111)); // LCD bias mode
        self.command(0b0010_0000); // use_extended_instruction_set = false
    }

    fn set_mode(&mut self, mode: Modes) {
        //self.command(0b0010_0000); // use_extended_instruction_set = false
        self.command(mode as u8); // set display control to normal mode: 0x0D for inverse
    }

    /// x must be 0..83
    /// y must be 0..5
    fn set_position(&mut self, x: u8, y: u8) {
        assert!(x < 84);
        assert!(y < 6);
        //self.command(0b0010_0000); // vertical_addressing = false
        self.command(0b0100_0000 | y);
        self.command(0b1000_0000 | x);
    }

    fn clear(&mut self) {
        self.set_position(0, 0);
        self.data(&[0u8; 6 * 84]);
        self.set_position(0, 0);
    }

    fn init(&mut self) {
        self.set_lcd_coefficients(56, 0, 4);
        self.set_mode(Modes::Normal);
        self.clear();
    }

    fn print_char(&mut self, c: u8) {
        let i = (c as usize) - 0x20;
        //self.set_function_set(false, false, false); // horizontal addressing
        self.data(&font::ASCII[i]);
        self.data(&[0u8]);
    }

    fn print(&mut self, s: &str) {
        for c in s.bytes() {
            self.print_char(c);
        }
    }

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; 6 * 84]) {
        self.command(0b0010_0010); // vertical_addressing = true
        self.set_position(0, 0);
        self.data(buffer);
        self.command(0b0010_0000); // vertical_addressing = false
        self.set_position(0, 0);
    }
}
