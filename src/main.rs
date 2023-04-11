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

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp_hal::{
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc
};
use esp_backtrace as _;
use esp_println::println;
use nb::block;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};



#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        // &mut system.peripheral_clock_control,
    );
    let mut timer0 = timer_group0.timer0;
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable watchdog timer
    wdt.disable();
    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    println!("Initialized timer group, rtc, wdt");

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c_led = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let i2c_sensors = I2C::new(
        peripherals.I2C1,
        io.pins.gpio32,
        io.pins.gpio33,
        10u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );


    println!("Initialized i2c");

    // Start timer (5 second interval)
    timer0.start(5u64.secs());

    println!("Starting timer");

    // Initialize display
    let interface = I2CDisplayInterface::new(i2c_led);
    println!("Connected display via i2c");

    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    println!("new ssd1306");

    match display.init() {
        Err(err) => println!("Display err {:?}", err),
        Ok(_) => ()
    };

    println!("Initialized display");

    // Specify different text styles
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_style_big = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    loop {
        // Fill display bufffer with a centered text with two lines (and two text
        // styles)
        Text::with_alignment(
            "vectos.net",
            display.bounding_box().center() + Point::new(0, 0),
            text_style_big,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            "Chip: ESP32",
            display.bounding_box().center() + Point::new(0, 14),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        // Write buffer to display
        display.flush().unwrap();
        // Clear display buffer
        display.clear();

        // Wait 5 seconds
        block!(timer0.wait()).unwrap();

        // Write single-line centered text "Hello World" to buffer
        Text::with_alignment(
            "Hello World!",
            display.bounding_box().center(),
            text_style_big,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        // Write buffer to display
        display.flush().unwrap();
        // Clear display buffer
        display.clear();

        // Wait 5 seconds
        block!(timer0.wait()).unwrap();
    }
}
