#[derive(Copy, Clone, Default)]
pub struct Calibration {
    dig_t: [u16; 3],
    dig_p: [u16; 9],
}

impl Calibration {
    pub fn from_bytes(bytes: &[u8; 24]) -> Self {
        let mut calibration: Self = Self { ..Default::default() };
        for i in 0..calibration.dig_t.len() {
            calibration.dig_t[i] = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
        }
        let bytes = &bytes[6..];
        for i in 0..calibration.dig_p.len() {
            calibration.dig_p[i] = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
        }
        calibration
    }

    #[inline]
    fn dig_t(self, index: usize) -> i32 {
        if index == 1 {
            self.dig_t[0] as i32
        } else {
            self.dig_t[index - 1] as i16 as i32
        }
    }

    #[inline]
    fn dig_p(self, index: usize) -> i32 {
        if index == 1 {
            self.dig_p[0] as i32
        } else {
            self.dig_p[index - 1] as i16 as i32
        }
    }
}

macro_rules! pow2 {
    ($x:expr) => {
        $x * $x
    };
}

#[derive(Copy, Clone)]
pub struct TemperatureFine(i32);

impl TemperatureFine {
    #[inline]
    pub fn degree_celsuis_x100(self) -> i32 {
        (self.0 * 5 + 128) >> 8
    }
}

#[derive(Copy, Clone)]
pub struct RawTemperature(i32);

impl RawTemperature {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self((bytes[0] as i32) << 12 | (bytes[1] as i32) << 4 | (bytes[2] as i32) >> 4)
    }

    pub fn t_fine(self, calib: &Calibration) -> TemperatureFine {
        let var1 = (((self.0 >> 3) - (calib.dig_t(1) << 1)) * calib.dig_t(2)) >> 11;
        let var2 = ((pow2!((self.0 >> 4) - calib.dig_t(1)) >> 12) * calib.dig_t(3)) >> 14;
        TemperatureFine(var1 + var2)
    }
}

#[derive(Copy, Clone)]
pub struct RawPressure(i32);

impl RawPressure {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self((bytes[0] as i32) << 12 | (bytes[1] as i32) << 4 | (bytes[2] as i32) >> 4)
    }

    // output in pa
    pub fn compensated(self, t_fine: TemperatureFine, calib: &Calibration) -> u32 {
        let mut var1 = (t_fine.0 >> 1) - 64000;
        let mut var2 = (pow2!(var1 >> 2) >> 11) * calib.dig_p(6);
        var2 = var2 + ((var1 * calib.dig_p(5)) << 1);
        var2 = (var2 >> 2) + (calib.dig_p(4) << 16);
        var1 = (((calib.dig_p(3) * (pow2!(var1 >> 2) >> 13)) >> 3)
            + ((calib.dig_p(2) * var1) >> 1))
            >> 18;
        var1 = ((32768 + var1) * calib.dig_p(1)) >> 15;
        if var1 == 0 {
            return 0;
        }
        let mut p = ((1048576 - self.0) - (var2 >> 12)) as u32 * 3125;
        if p < 0x8000000 {
            p = (p << 1) / var1 as u32;
        } else {
            p = (p / var1 as u32) * 2;
        }
        var1 = (calib.dig_p(9) * ((pow2!(p >> 3) >> 13) as i32)) >> 12;
        var2 = ((p >> 2) as i32 * calib.dig_p(8)) >> 13;
        (p as i32 + ((var1 + var2 + calib.dig_p(7)) >> 4)) as u32
    }

    // Pa*256, e.g. 24674867 represents 24674867 / 256 = 96386.2 Pa or 963.862 hPa
    pub fn i64_compensated(self, t_fine: TemperatureFine, calib: &Calibration) -> u32 {
        let mut var1 = (t_fine.0 - 128000) as i64;
        let mut var2 = pow2!(var1) * calib.dig_p(6) as i64 + ((var1 * calib.dig_p(5) as i64) << 17);
        var2 += (calib.dig_p(4) as i64) << 35;
        var1 =
            ((pow2!(var1) * calib.dig_p(3) as i64) >> 8) + ((var1 * calib.dig_p(2) as i64) << 12);
        var1 = (((1i64 << 47) + var1) * (calib.dig_p(1) as i64)) >> 33;
        if var1 == 0 {
            return 0;
        }
        let mut p = (1048576 - self.0) as i64;
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = (calib.dig_p(9) as i64 * pow2!(p >> 13)) >> 25;
        var2 = (calib.dig_p(8) as i64 * p) >> 19;
        (((p + var1 + var2) >> 8) + ((calib.dig_p(7) as i64) << 4)) as u32
    }
}

mod test {
    #[test]
    fn compensate() {
        use super::{Calibration, RawPressure, RawTemperature};

        let calibration_bytes = hex!(
            "99 6B 29 65 32 00 AF 92 7E D6 D0 0B
             C6 22 E2 FE F9 FF 8C 3C F8 C6 70 17"
        );
        let calibration = Calibration::from_bytes(&calibration_bytes);
        let raw_pressure = RawPressure::from_bytes(&hex!("4E 54 80"));
        let raw_temperature = RawTemperature::from_bytes(&hex!("8A BC 40"));
        let t_fine = raw_temperature.t_fine(&calibration);
        let temperature = t_fine.degree_celsuis_x100() / 100;
        assert_eq!(39, temperature);
        let pressure = raw_pressure.i64_compensated(t_fine, &calibration);
        assert_eq!(99792, pressure / 256);
        let pressure = raw_pressure.compensated(t_fine, &calibration);
        assert_eq!(99795, pressure);
    }
}
