#![no_std]

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[macro_use]
pub mod registers;
pub mod bus;
pub mod measurement;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::spi::{self, Phase, Polarity};

use bus::Bus;
use measurement::{Calibration, RawPressure, RawTemperature};
use registers::{PressureOversampling, Register, StandbyTime, TemperatureOversampling, ID, RESET};

pub const DEFAULT_SPI_MODE: spi::Mode =
    spi::Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnSecondTransition };
pub const ALTERNATE_SPI_MODE: spi::Mode =
    spi::Mode { polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition };

pub enum Mode {
    Sleep = 0b00,
    Forced = 0b01,
    Normal = 0b11,
}

pub struct BMP280<BUS> {
    bus: BUS,
}

impl<E, BUS: Bus<Error = E>> BMP280<BUS> {
    pub fn new(bus: BUS) -> Self {
        BMP280 { bus }
    }

    pub fn free(self) -> BUS {
        self.bus
    }

    pub fn verify(&mut self) -> Result<bool, E> {
        let id = self.bus.read(Register::Id)?;
        Ok(id == ID)
    }

    pub fn set_register(&mut self, reg: Register, offset: u8, len: u8, bits: u8) -> Result<(), E> {
        let mut value = self.bus.read(reg)?;
        let mask = (1u8 << len) - 1;
        value &= !(mask << offset);
        value |= (bits & mask) << offset;
        self.bus.write(reg, value)
    }

    pub fn reset<D: DelayMs<u8>>(&mut self, delay: &mut D) -> Result<(), E> {
        self.bus.write(Register::Reset, RESET)?;
        delay.delay_ms(2u8.into());
        Ok(())
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), E> {
        self.set_register(Register::ControlMeasurement, 0, 2, mode as u8)
    }

    pub fn set_pressure_oversampling(&mut self, value: PressureOversampling) -> Result<(), E> {
        self.set_register(Register::ControlMeasurement, 2, 3, value as u8)
    }

    pub fn set_temperature_oversampling(&mut self, v: TemperatureOversampling) -> Result<(), E> {
        self.set_register(Register::ControlMeasurement, 5, 3, v as u8)
    }

    pub fn set_standby_time(&mut self, standby_time: StandbyTime) -> Result<(), E> {
        self.set_register(Register::Config, 5, 2, standby_time as u8)
    }

    pub fn set_iir_filter(&mut self, filter_coefficient: u8) -> Result<(), E> {
        let value = match filter_coefficient {
            0..=1 => 0,
            2..=3 => 1,
            4..=7 => 2,
            8..=15 => 3,
            16..=255 => 4,
        };
        self.set_register(Register::Config, 2, 3, value)
    }

    pub fn read_calibration(&mut self) -> Result<Calibration, E> {
        let mut bytes = [0u8; 24];
        self.bus.reads(Register::Calib0, &mut bytes)?;
        Ok(Calibration::from_bytes(&bytes))
    }

    pub fn read_measurements(&mut self) -> Result<(RawPressure, RawTemperature), E> {
        let mut bytes = [0u8; 6];
        self.bus.reads(Register::PressureMsb, &mut bytes)?;
        Ok((RawPressure::from_bytes(&bytes), RawTemperature::from_bytes(&bytes[3..])))
    }
}
