

use esp_hal::ehal::blocking::delay::DelayMs;
use esp_hal::ehal::blocking::i2c::{Read, Write};
use crc_all::CrcAlgo;
use esp_println::println;
use lazy_static::lazy_static;

static CRC8_SEED: u8 = 0xffu8;
static CRC8_POLYNOMIAL: u8 = 0x31u8;

pub struct Ags02ma<I2C, D> {
    pub i2c: I2C,
    pub delay: D
}

#[derive(Debug)]
pub enum Ags02maError {
    BusWriteError,
    BusReadError,
    CrcError { expected: u8, actual: u8 }
}


impl <I2C, D> Ags02ma<I2C, D> where I2C : Read + Write, D : DelayMs<u16> {
    pub fn read_gas(&mut self) -> Result<u32, Ags02maError> {
        let res = self.execute(1500, &[0x20])?;
        Ok(res * 100)
    }

    pub fn read_tvoc(&mut self) -> Result<u32, Ags02maError> {
        let res = self.execute(1500, &[0x00])?;
        Ok(res & 0xffffff)
    }

    fn execute(&mut self, delay_ms: u16, cmd: &[u8]) -> Result<u32, Ags02maError> {
        lazy_static! {
            static ref CRC: CrcAlgo<u8> = CrcAlgo::<u8>::new(CRC8_POLYNOMIAL, 8, CRC8_SEED, 0x00, false);
        }

        let mut buf = [0u8; 5];
        self.i2c.write(0x1a, cmd).map_err(|_| Ags02maError::BusWriteError)?;
        self.delay.delay_ms(delay_ms);
        self.i2c.read(0x1a, &mut buf).map_err(|_| Ags02maError::BusReadError)?;

        let crc = &mut 0u8;
        CRC.init_crc(crc);
        let crc_res = CRC.update_crc(crc, &buf[0..4]);
        if crc_res != buf[4] {
            return Err(Ags02maError::CrcError { expected: buf[4], actual: crc_res });
        }

        let mut temp: u32 = buf[0] as u32;
        temp <<= 8;
        temp |= buf[1] as u32;
        temp <<= 8;
        temp |= buf[2] as u32;
        temp <<= 8;
        temp |= buf[3] as u32;

        return Ok(temp);
    }
}