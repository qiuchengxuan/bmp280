use core::convert::Infallible;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::i2c;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use super::registers::Register;

pub trait Bus {
    type Error;
    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error>;
    fn read(&mut self, reg: Register) -> Result<u8, Self::Error>;
    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum SpiError<WE, TE, OE> {
    WriteError(WE),
    TransferError(TE),
    OutputPinError(OE),
}

pub struct DummyOutputPin {}

impl OutputPin for DummyOutputPin {
    type Error = Infallible;
    fn set_high(&mut self) -> Result<(), Infallible> {
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
}

pub struct SpiBus<SPI, CS, D> {
    spi: SPI,
    cs: CS,
    delay: D,
}

impl<SPI, CS, D> SpiBus<SPI, CS, D>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
    D: DelayUs<u8>,
{
    pub fn new(spi: SPI, cs: CS, delay: D) -> Self {
        Self { spi, cs, delay }
    }

    pub fn free(self) -> (SPI, CS, D) {
        (self.spi, self.cs, self.delay)
    }
}

impl<WE, TE, OE, SPI, CS, D> SpiBus<SPI, CS, D>
where
    SPI: spi::Transfer<u8, Error = TE> + spi::Write<u8, Error = WE>,
    CS: OutputPin<Error = OE>,
{
    fn chip_select(&mut self, select: bool) -> Result<(), SpiError<WE, TE, OE>> {
        if select { self.cs.set_low() } else { self.cs.set_high() }
            .map_err(|e| SpiError::OutputPinError(e))
    }
}

impl<WE, TE, OE, SPI, CS, D> Bus for SpiBus<SPI, CS, D>
where
    SPI: spi::Transfer<u8, Error = TE> + spi::Write<u8, Error = WE>,
    CS: OutputPin<Error = OE>,
    D: DelayUs<u8>,
{
    type Error = SpiError<WE, TE, OE>;

    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error> {
        self.chip_select(true)?;
        let result = self.spi.write(&[reg as u8 & 0x7F, value]);
        self.chip_select(false)?;
        self.delay.delay_us(1);
        result.map_err(|e| SpiError::WriteError(e))
    }

    fn read(&mut self, reg: Register) -> Result<u8, Self::Error> {
        let mut value = [0u8];
        self.chip_select(true)?;
        self.spi.write(&[reg as u8 | 0x80]).map_err(|e| SpiError::WriteError(e))?;
        self.spi.transfer(&mut value).map_err(|e| SpiError::TransferError(e))?;
        self.chip_select(false)?;
        Ok(value[0])
    }

    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error> {
        self.chip_select(true)?;
        self.spi.write(&[reg as u8 | 0x80]).map_err(|e| SpiError::WriteError(e))?;
        self.spi.transfer(output).map_err(|e| SpiError::TransferError(e))?;
        self.chip_select(false)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum I2cError<WE, RE> {
    WriteError(WE),
    ReadError(RE),
}

pub struct I2cBus<I2C> {
    i2c: I2C,
    addr: I2cAddress,
}

#[derive(Copy, Clone)]
pub enum I2cAddress {
    SdoToGnd = 0x76,
    SdoToInterfaceSupplyVoltage = 0x77,
}

impl<I2C> I2cBus<I2C> {
    pub fn new(i2c: I2C, addr: I2cAddress) -> Self {
        Self { i2c, addr }
    }

    pub fn free(self) -> I2C {
        return self.i2c;
    }
}

impl<I2C, WE, RE> Bus for I2cBus<I2C>
where
    I2C: i2c::WriteRead<Error = RE> + i2c::Write<Error = WE>,
{
    type Error = I2cError<WE, RE>;

    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error> {
        let result = self.i2c.write(self.addr as u8, &[reg as u8, value]);
        result.map_err(|e| I2cError::WriteError(e))
    }

    fn read(&mut self, reg: Register) -> Result<u8, Self::Error> {
        let mut value = [0u8];
        self.i2c
            .write_read(self.addr as u8, &[reg as u8], &mut value)
            .map_err(|e| I2cError::ReadError(e))?;
        Ok(value[0])
    }

    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.addr as u8, &[reg as u8], output).map_err(|e| I2cError::ReadError(e))?;
        Ok(())
    }
}
