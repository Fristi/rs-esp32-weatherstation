//! I2C Display example
//!
//! This example prints some text on an SSD1306-based
//! display (via I2C)
//!
//! The following wiring is assumed:
//! - SDA => GPIO32
//! - SCL => GPIO33

#![no_std]
#![no_main]


mod ags02ma;
mod delayshare;
mod aht20;

use ags02ma::*;
use delayshare::*;

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp_hal::{clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, Delay};
use esp_backtrace as _;
use esp_hal::ehal::blocking::i2c::{Read, Write, WriteRead};
use esp_println::println;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use shared_bus::*;
use format_no_std::show;
use ssd1306::mode::BufferedGraphicsMode;
use aht20::*;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        // &mut system.peripheral_clock_control,
    );

    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable watchdog timer
    wdt.disable();
    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        30u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let bus = BusManagerSimple::new(i2c);

    let mut display= Ssd1306::new(
        I2CDisplayInterface::new(bus.acquire_i2c()),
        DisplaySize128x32,
        DisplayRotation::Rotate0
    ).into_buffered_graphics_mode();

    display.init().unwrap();

    let delay_share = DelayShare::new(&mut delay);
    let mut aht20 = Aht20::new(bus.acquire_i2c(), delay_share).unwrap();

    // aht20.reset().unwrap();
    aht20.reset_registers().unwrap();
    aht20.calibrate().unwrap();

    loop {
        let tvoc = read_tvoc(bus.acquire_i2c(), &mut delay).unwrap();
        let mut buffer_line = [0_u8; 20];
        let line = show(&mut buffer_line, format_args!("{:?} bbp", tvoc)).unwrap();

        write_display(&mut display, "Gas reading", line);
        delay.delay_ms(2_000_u32);

        let res = read_res(bus.acquire_i2c(), &mut delay).unwrap();
        let line = show(&mut buffer_line, format_args!("{:?} ohm", res)).unwrap();

        write_display(&mut display, "Gas resistance", line);
        delay.delay_ms(2_000_u32);

        let mut aht20 = Aht20::new(bus.acquire_i2c(), DelayShare::new(&mut delay)).unwrap();

        let (hum, temp) = aht20.read().unwrap();
        let line = show(&mut buffer_line, format_args!("{:.1} RH% {:.1} C", hum.rh(), temp.celsius())).unwrap();

        write_display(&mut display, "Weather", line);
        delay.delay_ms(2_000_u32);
    }
}

fn read_res<I>(i2c: I, delay: &mut Delay) -> Result<u32, Ags02maError> where I : Write + Read {
    let delay_share = DelayShare::new(delay);
    let mut ags02ma = Ags02ma { i2c: i2c, delay: delay_share };

    ags02ma.read_gas()
}

fn read_tvoc<I>(i2c: I, delay: &mut Delay) -> Result<u32, Ags02maError> where I : Write + Read {
    let delay_share = DelayShare::new(delay);
    let mut ags02ma = Ags02ma { i2c: i2c, delay: delay_share };

    ags02ma.read_tvoc()
}

fn write_display<I>(display: &mut Ssd1306<I2CInterface<I>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>, big_text: &str, small_text: &str) where I : Write {

    // Specify different text styles
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_style_big = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    // Fill display bufffer with a centered text with two lines (and two text
    // styles)
    Text::with_alignment(
        big_text,
        display.bounding_box().center() + Point::new(0, 0),
        text_style_big,
        Alignment::Center,
    )
        .draw(display)
        .unwrap();

    Text::with_alignment(
        small_text,
        display.bounding_box().center() + Point::new(0, 14),
        text_style,
        Alignment::Center,
    )
        .draw(display)
        .unwrap();

    // Write buffer to display
    display.flush().unwrap();
    // Clear display buffer
    display.clear();
}