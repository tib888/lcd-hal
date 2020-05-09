use embedded_hal::{
    blocking::delay::DelayMs, 
    digital::v2::OutputPin
};

use super::{Modes, Hx1230, Hx1230Base};

use crate::{font, Display};

pub struct Hx1230Gpio<CLK, DIN, CS> {
    clk: CLK, //clock
    din: DIN, //data
    cs: CS,   //chip select
}

impl<CLK, DIN, CS, E> Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin<Error = E>,
    DIN: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    pub fn new<RST, DELAY>(
        clk: CLK,
        din: DIN,
        cs: CS,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<Hx1230Gpio<CLK, DIN, CS>, E>
    where
        RST: OutputPin<Error = E>,
        DELAY: DelayMs<u8>,
    {
        // Start by reseting the LCD controller
        rst.set_high()?;
        delay.delay_ms(50);
        rst.set_low()?;
        delay.delay_ms(5);
        rst.set_high()?; // take it out of reset
        delay.delay_ms(10);

        // turn on and initialize the display:
        let mut hx = Hx1230Gpio { clk, din, cs };
        hx.init()?;
        Ok(hx)
    }

    fn send(&mut self, byte: u8) -> Result<(), E> {
        //MSB first
        for bit in (0..8).rev() {
            if (byte & (1 << bit)) != 0 {
                self.din.set_high()?;
            } else {
                self.din.set_low()?;
            }

            self.clk.set_high()?; // toggle clock
            self.clk.set_low()?; // high->low transition latches data
        }
        Ok(())
    }
}

impl<CLK, DIN, CS, E> Hx1230Base for Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin<Error = E>,
    DIN: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    type Error = E;

    fn command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        self.cs.set_low()?;

        self.din.set_low()?; // set d/c = 0 means command
        self.clk.set_high()?; // toggle clock
        self.clk.set_low()?; // high->low transition latches data

        self.send(cmd)?;
        self.cs.set_high()?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low()?;
        //self.dc.set_high()?;

        for byte in data {
            self.din.set_high()?; // set d/c = 1 means data
            self.clk.set_high()?; // toggle clock
            self.clk.set_low()?; // high->low transition latches data

            self.send(*byte)?;
        }
        self.cs.set_high()?;
        Ok(())
    }
}

impl<CLK, DIN, CS, E> Display for Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin<Error = E>,
    DIN: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    type Error = E;

    /// x must be 0..95
    /// y must be 0..7
    fn set_position(&mut self, x: u8, y: u8) -> Result<(), Self::Error> {
        assert!(x < 96);
        assert!(y < 8);
        // set Y
        //10110YYY set page [0..7]
        self.command(0xb0 | y)?;
        // set X MSB
        //00010XXX column high 3 bits
        self.command(0x10 | (x >> 4))?;
        // set X LSB
        //0000XXXX column low 4 bits
        self.command(0x00 | (x & 0xf))?;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.set_position(0, 0)?;
        self.data(&[0u8; 9 * 96])?; //clear the last half row too
        self.set_position(0, 0)?;
        Ok(())
    }

    fn print_char(&mut self, c: u8) -> Result<(), Self::Error> {
        let i = (c as usize) - 0x20;
        self.data(&font::ASCII[i])?;
        self.data(&[0u8])?;
        Ok(())
    }

    fn get_pixel_resolution(&self) -> (u8, u8) {
        (96, 68)
    }

    fn get_char_resolution(&self) -> (u8, u8) {
        (16, 8)
    }
}

impl<T: Hx1230Base> Hx1230 for T {
    type Error = T::Error;

    /// contrast < 32
    fn set_contrast(&mut self, contrast: u8) -> Result<(), Self::Error> {
        assert!(contrast < 32);
        //100***** set contrast
        self.command(0b100_00000 | contrast)?;
        Ok(())
    }

    fn set_mode(&mut self, mode: Modes) -> Result<(), Self::Error> {
        //1010010* set all pixel on
        //1010011* set inverse display

        match mode {
            Modes::Blank => {
                self.command(0b10100111)?;
                self.command(0b10100101)?; //all on, inverse
            }
            Modes::Normal => {
                self.command(0b10100110)?;
                self.command(0b10100100)?; //normal, not inverse
            }
            Modes::Filled => {
                self.command(0b10100110)?;
                self.command(0b10100101)?; //all on, not inverse
            }
            Modes::Inverse => {
                self.command(0b10100111)?;
                self.command(0b10100100)?; //normal, inverse
            }
        }
        Ok(())
    }

    fn flip_horizontal(&mut self, flip: bool) -> Result<(), Self::Error> {
        // set SEG direction (A1 to flip horizontal)
        self.command(if flip { 0xa1 } else { 0xa8 })
    }

    fn flip_vertical(&mut self, flip: bool) -> Result<(), Self::Error> {
        // set COM direction (C8 to flip vert)
        self.command(if flip { 0xc8 } else { 0xc0 })
    }

    fn init(&mut self) -> Result<(), Self::Error> {
        // turn on and initialize the display:
        self.command(0b0010_1111)?; //0010**** set power
        self.set_contrast(0)?; //0x90
        self.set_mode(Modes::Normal)?; //0xa6, 0xa4
        self.command(0b1010_1111)?; //1010111* enable display
        self.command(0b0100_0000)?; //01****** set scan start line
        Ok(())
    }
}
