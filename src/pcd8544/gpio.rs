use embedded_hal::{
    blocking::delay::DelayMs,
    digital::v2::OutputPin
};

use super::{Pcd8544, Pcd8544Base};

pub struct Pcd8544Gpio<CLK, DIN, DC, CS> {
    clk: CLK,
    din: DIN,
    dc: DC,
    cs: CS,
}

impl<CLK, DIN, DC, CS, E> Pcd8544Gpio<CLK, DIN, DC, CS>
where
    CLK: OutputPin<Error = E>,
    DIN: OutputPin<Error = E>,
    DC: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
    
{
    pub fn new<RST, DELAY>(clk: CLK, din: DIN, dc: DC, cs: CS, rst: &mut RST, delay: &mut DELAY) -> Result<Pcd8544Gpio<CLK, DIN, DC, CS>, E>
        where RST : OutputPin<Error = E>, DELAY: DelayMs<u8>
    {
        rst.set_low()?;
        delay.delay_ms(10);
        rst.set_high()?;

        let mut pcd = Pcd8544Gpio { clk, din, dc, cs };
        pcd.init()?;
        Ok(pcd)
    }

    fn send(&mut self, byte: u8) -> Result<(), E> {
        for bit in (0..8).rev() {
            if (byte & (1 << bit)) != 0 {
                self.din.set_high()?;
            } else {
                self.din.set_low()?;
            }

            self.clk.set_high()?;
            self.clk.set_low()?;            
        };
        
        Ok(())
    }
}

impl<CLK, DIN, DC, CS, E> Pcd8544Base for Pcd8544Gpio<CLK, DIN, DC, CS>
where
    CLK: OutputPin<Error = E>,
    DIN: OutputPin<Error = E>,
    DC: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    type Error = E;

    fn command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        self.dc.set_low()?;
        self.cs.set_low()?;
        self.send(cmd)?;
        self.cs.set_high()?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high()?;
        self.cs.set_low()?;
        for byte in data {
            self.send(*byte)?;
        }
        self.cs.set_high()?;
        Ok(())
    }
}
