pub const ID: u8 = 0x58;

pub const RESET: u8 = 0xb6;

#[derive(Copy, Clone)]
pub enum Register {
    Calib0 = 0x88,
    Id = 0xd0,
    Reset = 0xe0,
    Status = 0xf3,
    ControlMeasurement = 0xf4,
    Config = 0xf5,
    PressureMsb = 0xf7,
    PressureLsb = 0xf8,
    PressureXLsb = 0xf9,
    TemperatureMsb = 0xfa,
    TemperatureLsb = 0xfb,
    TemperatureXLsb = 0xfc,
}

pub enum TemperatureOversampling {
    Skipped = 0x0,
    UltraLowPower = 0x1,
    LowPower = 0x2,
    StandardResolution = 0x3,
    HighResolution = 0x4,
    UltraHighResolution = 0x5,
}

#[macro_export]
macro_rules! temperature_resolution {
    (16bit/0.0050dC) => {
        TemperatureOversampling::UltraLowPower
    };
    (17bit/0.0025dC) => {
        TemperatureOversampling::LowPower
    };
    (18bit/0.0012dC) => {
        TemperatureOversampling::StandardResolution
    };
    (19bit/0.0006dC) => {
        TemperatureOversampling::HighResolution
    };
    (20bit/0.0003dC) => {
        TemperatureOversampling::UltraHighResolution
    };
}

pub enum PressureOversampling {
    Skipped = 0x0,
    UltraLowPower = 0x1,
    LowPower = 0x2,
    StandardResolution = 0x3,
    HighResolution = 0x4,
    UltraHighResolution = 0x5,
}

#[macro_export]
macro_rules! pressure_resolution {
    (16bit/2.62Pa) => {
        PressureOversampling::UltraLowPower
    };
    (17bit/1.31Pa) => {
        PressureOversampling::LowPower
    };
    (18bit/0.66Pa) => {
        PressureOversampling::StandardResolution
    };
    (19bit/0.33Pa) => {
        PressureOversampling::HighResolution
    };
    (20bit/0.16Pa) => {
        PressureOversampling::UltraHighResolution
    };
}

pub enum StandbyTime {
    Hertz2000 = 0,
    Hertz16,
    Hertz8,
    Hertz4,
    Hertz2,
    Second,
    Second2,
    Second4,
}
