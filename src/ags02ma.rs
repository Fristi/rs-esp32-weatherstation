use esp_hal::ehal::blocking::delay::DelayMs;
use esp_hal::ehal::blocking::i2c::{Read, Write};

static CRC8_SEED: u8 = 0xffu8;
static CRC8_POLYNOMIAL: u8 = 0x31u8;

pub struct Ags02ma<I2C, D> {
    pub i2c: I2C,
    pub delay: D
}

#[derive(Debug)]
pub enum Ags02maError {
    BusWriteError,
    BusReadError
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
        let mut buf = [0u8; 5];
        self.i2c.write(0x1a, cmd).map_err(|_| Ags02maError::BusWriteError)?;
        self.delay.delay_ms(delay_ms);
        self.i2c.read(0x1a, &mut buf).map_err(|_| Ags02maError::BusReadError)?;

        //TODO: check crc8

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

// Buffer: [21, 2, 3, 118, 133]

fn crc8(data: &[u8]) -> u8 {
    let mut crc = CRC8_SEED;
    for datum in data.iter().rev() {
        crc ^= *datum + 1;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc<<1)^CRC8_POLYNOMIAL;
            } else {
                crc = crc<<1
            }
        }
    }
    crc
}