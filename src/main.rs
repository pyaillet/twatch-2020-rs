#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]

mod dprint;

use core::{self, panic::PanicInfo};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    image::*,
    pixelcolor::Rgb565,
    prelude::*,
    style::{TextStyle, TextStyleBuilder},
};
use embedded_hal::digital::v2::OutputPin;
use esp32;
use esp32_hal::{
    clock_control::{sleep, ClockControl, XTAL_FREQUENCY_AUTO},
    dport::Split,
    gpio::{Gpio18, Gpio19, Gpio5, Output, PushPull},
    i2c,
    prelude::*,
    serial::{config::Config, Pins, Serial},
    spi::{self, SPI},
    target,
    timer::Timer,
};

use axp20x;
use display_interface_spi::SPIInterfaceNoCS;
use st7789::{Orientation, ST7789};

const BLINK_HZ: Hertz = Hertz(1);

struct NoPin {}

impl Default for NoPin {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
enum Infalible {}

impl OutputPin for NoPin {
    /// Error type
    type Error = Infalible;

    /// Drives the pin low
    ///
    /// *NOTE* the actual electrical state of the pin may not actually be low, e.g. due to external
    /// electrical sources
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Drives the pin high
    ///
    /// *NOTE* the actual electrical state of the pin may not actually be high, e.g. due to external
    /// electrical sources
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

macro_rules! digit_to_u8 {
    ($num:expr, $rank:expr) => {
        if ($num >= $rank) {
            ((($num / $rank) % 10) + 48)
        } else {
            32
        }
    };
}

macro_rules! percentage_to_str {
    ($num:expr) => {
        core::str::from_utf8(&[
            digit_to_u8!($num, 100),
            digit_to_u8!($num, 10),
            digit_to_u8!($num, 1),
            32,
            37,
        ])
        .unwrap()
    };
}

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().expect("Failed to obtain Peripherals");

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    let clkcntrl = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);
    watchdog0.disable();
    watchdog1.disable();

    let pins = dp.GPIO.split();

    // Use UART1 as example: will cause dprintln statements not to be printed
    let serial: Serial<_, _, _> = Serial::new(
        dp.UART1,
        Pins {
            tx: pins.gpio1,
            rx: pins.gpio3,
            cts: None,
            rts: None,
        },
        Config {
            // default configuration is 19200 baud, 8 data bits, 1 stop bit & no parity (8N1)
            baudrate: 115200.Hz(),
            ..Config::default()
        },
        clkcntrl_config,
    )
    .unwrap();

    let mut gpio_backlight = pins.gpio12.into_push_pull_output();
    let sclk = pins.gpio18.into_push_pull_output();
    let sdo = pins.gpio19.into_push_pull_output();
    let cs = pins.gpio5.into_push_pull_output();

    // Official ili9341 spec is 10MHz, but overdrive up to 80MHz actually works.
    // 26MHz chosen here: will be 26MHz when using 26MHz crystal, 20MHz when using 40MHz crystal,
    // due to integer clock division.
    // Faster is no use as the cpu is not keeping up with the embedded_graphics library.
    let spi: SPI<
        esp32::SPI2,
        Gpio18<Output<PushPull>>,
        Gpio19<Output<PushPull>>,
        Gpio19<Output<PushPull>>,
        Gpio5<Output<PushPull>>,
    > = SPI::<esp32::SPI2, _, _, _, _>::new(
        dp.SPI2,
        spi::Pins {
            sclk,
            sdo,
            sdi: None,
            cs: Some(cs),
        },
        spi::config::Config {
            baudrate: 80.MHz().into(),
            bit_order: spi::config::BitOrder::MSBFirst,
            data_mode: spi::config::MODE_3,
        },
        clkcntrl_config,
    )
    .unwrap();

    let i2c0 = i2c::I2C::new(
        dp.I2C0,
        i2c::Pins {
            sda: pins.gpio21,
            scl: pins.gpio22,
        },
        400_000,
        &mut dport,
    );
    let mut axp = axp20x::AXP20X::new(i2c0);
    axp.init(&mut esp32_hal::delay::Delay::new());

    gpio_backlight.set_low().unwrap();

    let gpio_dc = pins.gpio27.into_push_pull_output();

    let spi_if = SPIInterfaceNoCS::new(spi, gpio_dc);

    // create driver
    let mut display = ST7789::new(spi_if, NoPin::default(), 240, 240);

    // draw image on black background
    // initialize
    display.init(&mut esp32_hal::delay::Delay::new()).unwrap();
    // set default orientation
    display.set_orientation(Orientation::Portrait).unwrap();

    dprintln!("\n\nESP32 Started\n\n");

    let mut down = true;
    match gpio_backlight.set_high() {
        Ok(()) => dprint!("Ok High\r\n"),
        Err(_e) => dprint!("KO\r\n"),
    };

    let raw_image_data: ImageRawLE<Rgb565> =
        ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86, 64);
    let ferris = Image::new(&raw_image_data, Point::new(34, 8));
    // draw image on black background
    match display.clear(Rgb565::BLACK) {
        Ok(()) => dprint!("Ok\r\n"),
        Err(_e) => dprint!("KO\r\n"),
    };
    match ferris.draw(&mut display) {
        Ok(()) => dprint!("Ok\r\n"),
        Err(_e) => dprint!("KO\r\n"),
    };

    // Create a new text style
    let style = TextStyleBuilder::new(Font6x8)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    loop {
        dprint!("Ok !\r\n");
        let charge = match axp.is_charging() {
            Ok(true) => "Charging... ",
            Ok(false) => "Not charging",
            _ => "Error",
        };
        let battery = match axp.is_battery_connect() {
            Ok(true) => "Battery connected    ",
            Ok(false) => "Battery not connected",
            _ => "Error",
        };
        let percentage = match axp.get_battery_percentage() {
            Ok(p) => p,
            Err(_) => 101,
        };
        dprint!("{}%\r\n", percentage);
        Text::new(charge, Point::new(40, 75))
            .into_styled(style)
            .draw(&mut display);
        Text::new(battery, Point::new(40, 85))
            .into_styled(style)
            .draw(&mut display);
        Text::new(percentage_to_str!(percentage), Point::new(40, 95))
            .into_styled(style)
            .draw(&mut display);
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("Panic: {:?}\r\n", info);
    loop {}
}
