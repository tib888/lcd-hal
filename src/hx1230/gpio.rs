use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::OutputPin;

use super::super::{font, Display};
use super::{Hx1230, Hx1230Base, Modes};

pub struct Hx1230Gpio<CLK, DIN, CS> {
    clk: CLK, //clock
    din: DIN, //data
    cs: CS,   //chip select
}

impl<CLK, DIN, CS> Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    CS: OutputPin,
{
    pub fn new(
        clk: CLK,
        din: DIN,
        cs: CS,
        rst: &mut OutputPin,
        delay: &mut DelayMs<u8>,
    ) -> Hx1230Gpio<CLK, DIN, CS> {
        // Start by reseting the LCD controller
        rst.set_high();
        delay.delay_ms(50);
        rst.set_low();
        delay.delay_ms(5);
        rst.set_high(); // take it out of reset
        delay.delay_ms(10);

        // turn on and initialize the display:
        let mut hx = Hx1230Gpio { clk, din, cs };
        hx.init();
        hx
    }

    fn send(&mut self, byte: u8) {
        //MSB first
        for bit in (0..8).rev() {
            if (byte & (1 << bit)) != 0 {
                self.din.set_high();
            } else {
                self.din.set_low();
            }

            self.clk.set_high(); // toggle clock
            self.clk.set_low(); // high->low transition latches data
        }
    }
}

impl<CLK, DIN, CS> Hx1230Base for Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    CS: OutputPin,
{
    fn command(&mut self, cmd: u8) {
        self.cs.set_low();

        self.din.set_low(); // set d/c = 0 means command
        self.clk.set_high(); // toggle clock
        self.clk.set_low(); // high->low transition latches data

        self.send(cmd);
        self.cs.set_high();
    }

    fn data(&mut self, data: &[u8]) {
        self.cs.set_low();
        //self.dc.set_high();

        for byte in data {
            self.din.set_high(); // set d/c = 1 means data
            self.clk.set_high(); // toggle clock
            self.clk.set_low(); // high->low transition latches data

            self.send(*byte);
        }
        self.cs.set_high();
    }
}

impl<CLK, DIN, CS> Display for Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    CS: OutputPin,
{
    /// x must be 0..95
    /// y must be 0..7
    fn set_position(&mut self, x: u8, y: u8) {
        assert!(x < 96);
        assert!(y < 8);
        // set Y
        //10110YYY set page [0..7]
        self.command(0xb0 | y);
        // set X MSB
        //00010XXX column high 3 bits
        self.command(0x10 | (x >> 4));
        // set X LSB
        //0000XXXX column low 4 bits
        self.command(0x00 | (x & 0xf));
    }

    fn clear(&mut self) {
        self.set_position(0, 0);
        self.data(&[0u8; 8 * 96]);
        self.set_position(0, 0);
    }

    fn print_char(&mut self, c: u8) {
        let i = (c as usize) - 0x20;
        self.data(&font::ASCII[i]);
        self.data(&[0u8]);
    }

    fn print(&mut self, s: &str) {
        for c in s.bytes() {
            self.print_char(c);
        }
    }

    fn get_pixel_resolution() -> (u8, u8) {
        (96, 68)
    }

    fn get_char_resolution() -> (u8, u8) {
        (16, 8)
    }
}

impl<CLK, DIN, CS> Hx1230 for Hx1230Gpio<CLK, DIN, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    CS: OutputPin,
{
    /// contrast < 32
    fn set_contrast(&mut self, contrast: u8) {
        assert!(contrast < 32);
        //100***** set contrast
        self.command(0b100_00000 | contrast);
    }

    fn set_mode(&mut self, mode: Modes) {
        //1010010* set all pixel on
        //1010011* set inverse display

        match mode {
            Modes::Blank => {
                self.command(0b10100111);
                self.command(0b10100101); //all on, inverse
            }
            Modes::Normal => {
                self.command(0b10100110);
                self.command(0b10100100); //normal, not inverse
            }
            Modes::Filled => {
                self.command(0b10100110);
                self.command(0b10100101); //all on, not inverse
            }
            Modes::Inverse => {
                self.command(0b10100111);
                self.command(0b10100100); //normal, inverse
            }
        }
    }

    fn flip_horizontal(&mut self, flip: bool) {
        // set SEG direction (A1 to flip horizontal)
        self.command(if flip { 0xa1 } else { 0xa8 });
    }

    fn flip_vertical(&mut self, flip: bool) {
        // set COM direction (C8 to flip vert)
        self.command(if flip { 0xc8 } else { 0xc0 });
    }

    fn init(&mut self) {
        // turn on and initialize the display:
        self.command(0b0010_1111); //0010**** set power
        self.set_contrast(0); //0x90
        self.set_mode(Modes::Normal); //0xa6, 0xa4
        self.command(0b1010_1111); //1010111* enable display
        self.command(0b0100_0000); //01****** set scan start line
        self.clear(); //this is included in clear: self.set_position(0, 0); 0xb0, 0x10, 0x00
    }
}
