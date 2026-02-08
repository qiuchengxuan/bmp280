use core::convert::Infallible;

use embedded_hal::digital::ErrorType;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use embedded_hal::spi::SpiDevice;

use super::registers::Register;

pub trait Bus {
    type Error;
    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error>;
    fn read(&mut self, reg: Register) -> Result<u8, Self::Error>;
    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum SpiError<TE, OE> {
    TransferError(TE),
    OutputPinError(OE),
}

pub struct DummyOutputPin {}

impl ErrorType for DummyOutputPin {
    type Error = Infallible;
}

impl OutputPin for DummyOutputPin {
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
    SPI: SpiDevice<u8>,
    CS: OutputPin,
    D: embedded_hal::delay::DelayNs,
{
    pub fn new(spi: SPI, cs: CS, delay: D) -> Self {
        Self { spi, cs, delay }
    }

    pub fn free(self) -> (SPI, CS, D) {
        (self.spi, self.cs, self.delay)
    }
}

impl<TE, OE, SPI, CS, D> SpiBus<SPI, CS, D>
where
    SPI: SpiDevice<u8, Error = TE>,
    CS: OutputPin<Error = OE>,
{
    fn chip_select(&mut self, select: bool) -> Result<(), SpiError<TE, OE>> {
        if select { self.cs.set_low() } else { self.cs.set_high() }
            .map_err(|e| SpiError::OutputPinError(e))
    }
}

impl<TE, OE, SPI, CS, D> Bus for SpiBus<SPI, CS, D>
where
    SPI: SpiDevice<u8, Error = TE>,
    CS: OutputPin<Error = OE>,
    D: embedded_hal::delay::DelayNs,
{
    type Error = SpiError<TE, OE>;

    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error> {
        self.chip_select(true)?;
        let result = self.spi.write(&[reg as u8 & 0x7F, value]);
        self.chip_select(false)?;
        self.delay.delay_us(1);
        result.map_err(|e| SpiError::TransferError(e))
    }

    fn read(&mut self, reg: Register) -> Result<u8, Self::Error> {
        let mut value = [0u8];
        self.chip_select(true)?;
        self.spi.write(&[reg as u8 | 0x80]).map_err(|e| SpiError::TransferError(e))?;
        self.spi.read(&mut value).map_err(|e| SpiError::TransferError(e))?;
        self.chip_select(false)?;
        Ok(value[0])
    }

    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error> {
        self.chip_select(true)?;
        self.spi.write(&[reg as u8 | 0x80]).map_err(|e| SpiError::TransferError(e))?;
        self.spi.read(output).map_err(|e| SpiError::TransferError(e))?;
        self.chip_select(false)?;
        Ok(())
    }
}

#[derive(Debug)]

pub struct I2cBus<I2C> {
    i2c: I2C,
    addr: I2cAddress,
}

#[derive(Copy, Clone, Debug)]
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

impl<I2C, E> Bus for I2cBus<I2C>
where
    I2C: I2c<Error = E>,
{
    type Error = E;

    fn write(&mut self, reg: Register, value: u8) -> Result<(), Self::Error> {
        self.i2c.write(self.addr as u8, &[reg as u8, value])
    }

    fn read(&mut self, reg: Register) -> Result<u8, Self::Error> {
        let mut value = [0u8];
        self.i2c.write_read(self.addr as u8, &[reg as u8], &mut value)?;
        Ok(value[0])
    }

    fn reads(&mut self, reg: Register, output: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.addr as u8, &[reg as u8], output)
    }
}
