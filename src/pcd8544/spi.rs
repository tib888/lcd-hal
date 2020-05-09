use embedded_hal::{
    blocking::{ 
        delay::DelayMs, 
        spi::Write
    },
    digital::v2::OutputPin
};

use super::{Pcd8544, Pcd8544Base};

pub struct Pcd8544Spi<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

impl<SPI, DC, CS, E> Pcd8544Spi<SPI, DC, CS>
where
    SPI: Write<u8>,
    DC: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    pub fn new<RST, DELAY>(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<Pcd8544Spi<SPI, DC, CS>, E> 
    where RST: OutputPin<Error = E>, DELAY: DelayMs<u8>
    {
        rst.set_low()?;
        delay.delay_ms(10);
        rst.set_high()?;

        let mut pcd = Pcd8544Spi { spi, dc, cs };
        pcd.init()?;
        Ok(pcd)
    }
}

impl<SPI, DC, CS, E> Pcd8544Base for Pcd8544Spi<SPI, DC, CS>
where
    SPI: Write<u8>,
    DC: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
{
    type Error = E;

    fn command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        self.dc.set_low()?;
        self.cs.set_low()?;
        let _ = self.spi.write(&[cmd]);
        self.cs.set_high()?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high()?;
        self.cs.set_low()?;
        let _ = self.spi.write(data);
        self.cs.set_high()?;
        Ok(())
    }
}
