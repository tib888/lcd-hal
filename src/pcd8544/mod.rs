pub mod gpio;
pub mod spi;

use super::font;
use super::Display;

pub trait Pcd8544Base {
    type Error;
    fn command(&mut self, cmd: u8) -> Result<(), Self::Error>;
    fn data(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

pub enum Modes {
    Blank = 0b0001000,
    Normal = 0b0001100,
    Filled = 0b0001001,
    Inverse = 0b0001101,
}

pub trait Pcd8544 {
    type Error;

    /// voltage_coefficient < 90 => VLCD = 3.06 + voltage_coefficient * 0.06; VLCD must be less than 8.5V
    /// temp_coefficient < 4
    /// bias < 8 => Vbias = 1/(bias + 4) * VLCD
    fn set_lcd_coefficients(&mut self, voltage_coefficient: u8, temp_coefficient: u8, bias: u8) -> Result<(), Self::Error>;

    fn set_mode(&mut self, mode: Modes) -> Result<(), Self::Error>;

    fn init(&mut self) -> Result<(), Self::Error>;

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; 6 * 84]) -> Result<(), Self::Error>;
}

impl<T: Pcd8544Base> Display for T {
    type Error = T::Error;
    /// x must be 0..83
    /// y must be 0..5
    fn set_position(&mut self, x: u8, y: u8) -> Result<(), Self::Error> {
        assert!(x < 84);
        assert!(y < 6);
        //self.command(0b0010_0000); // vertical_addressing = false
        self.command(0b0100_0000 | y)?;
        self.command(0b1000_0000 | x)
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.set_position(0, 0)?;
        self.data(&[0u8; 6 * 84])?;
        self.set_position(0, 0)?;
        Ok(())
    }

    fn print_char(&mut self, c: u8) -> Result<(), Self::Error> {
        let i = (c as usize) - 0x20;
        //self.set_function_set(false, false, false); // horizontal addressing
        self.data(&font::ASCII[i])?;
        self.data(&[0u8])?;
        Ok(())
    }

    fn get_pixel_resolution(&self) -> (u8, u8) {
        (84, 48)
    }

    fn get_char_resolution(&self) -> (u8, u8) {
        (14, 6)
    }
}

impl<T: Pcd8544Base> Pcd8544 for T {
    type Error = T::Error;
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
    fn set_lcd_coefficients(&mut self, voltage_coefficient: u8, temp_coefficient: u8, bias: u8) -> Result<(), Self::Error> {
        assert!(voltage_coefficient < 91);
        assert!(temp_coefficient < 4);
        assert!(bias < 8);
        self.command(0b0010_0001)?; // use_extended_instruction_set = true
        self.command(0b1000_0000 | (voltage_coefficient & 0b0111_1111))?; // try 0x31 (for 3.3V red SparkFun), 0x38 (for 3.3V blue SparkFun), 0x3F if your display is too dark, or 0 to 90 if experimenting
        self.command(0b0000_0100 | (temp_coefficient & 0b0000_0011))?; // set temp coefficient
        self.command(0b0001_0000 | (bias & 0b0000_0111))?; // LCD bias mode
        self.command(0b0010_0000)?; // use_extended_instruction_set = false
        Ok(())
    }

    fn set_mode(&mut self, mode: Modes) -> Result<(), Self::Error> {
        //self.command(0b0010_0000); // use_extended_instruction_set = false
        self.command(mode as u8) // set display control to normal mode: 0x0D for inverse
    }

    fn init(&mut self) -> Result<(), Self::Error> {
        self.set_lcd_coefficients(56, 0, 4)?;
        self.set_mode(Modes::Normal)?;
        self.clear()
    }

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; 6 * 84]) -> Result<(), Self::Error> {
        self.command(0b0010_0010)?; // vertical_addressing = true
        self.set_position(0, 0)?;
        self.data(buffer)?;
        self.command(0b0010_0000)?; // vertical_addressing = false
        self.set_position(0, 0)
    }
}