use esp_hal::Delay;
use esp_hal::ehal::blocking::delay::DelayMs;

pub struct DelayShare<'a> {
    delay_bit: &'a mut Delay,
}

impl<'a> DelayShare<'a> {
    pub fn new(delay_bit: &'a mut Delay) -> Self {
        Self { delay_bit }
    }
}
impl<'a> DelayMs<u16> for DelayShare<'a> {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_bit.delay_ms(ms)
    }
}