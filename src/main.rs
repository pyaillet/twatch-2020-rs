#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]

use core::{self, panic::PanicInfo};
use embedded_graphics::{
    draw_target::DrawTarget,
    image::*,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use esp32_hal::{prelude::*, target};

use twatch::{self, dprint, dprintln, TWatchError};

macro_rules! digit_to_u8 {
    ($num:expr, $rank:expr) => {
        if $num >= $rank || $rank == 1 {
            ((($num / $rank) % 10) + 48) as u8
        } else {
            32
        }
    };
}

macro_rules! number_to_str {
    ($num:expr) => {
        core::str::from_utf8(&[
            digit_to_u8!($num, 1000000),
            digit_to_u8!($num, 100000),
            digit_to_u8!($num, 10000),
            digit_to_u8!($num, 1000),
            digit_to_u8!($num, 100),
            digit_to_u8!($num, 10),
            digit_to_u8!($num, 1),
        ])
        .unwrap()
    };
}

macro_rules! u8_to_str {
    ($num:expr) => {
        core::str::from_utf8(&[
            digit_to_u8!($num, 100),
            digit_to_u8!($num, 10),
            digit_to_u8!($num, 1),
        ])
        .unwrap()
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

fn display_debug(twatch: &mut twatch::TWatch) -> Result<(), TWatchError> {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let charge = if twatch
        .pmu
        .is_charging()
        .map_err(|_e| TWatchError::DisplayError)?
    {
        "Charging... "
    } else {
        "Not charging"
    };
    let battery = if twatch
        .pmu
        .is_battery_connect()
        .map_err(|_e| TWatchError::DisplayError)?
    {
        "Battery connected    "
    } else {
        "Battery not connected"
    };
    let percentage = twatch
        .get_battery_percentage()
        .map_err(|_e| TWatchError::DisplayError)?;
    dprint!("{}%\r\n", percentage);
    Text::new(charge, Point::new(40, 75), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    Text::new(battery, Point::new(40, 85), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    Text::new(percentage_to_str!(percentage), Point::new(40, 95), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    twatch::sleep(1000000_u32.us());
    twatch
        .display
        .clear(Rgb565::BLACK)
        .map_err(|_e| TWatchError::DisplayError)?;
    Ok(())
}

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().expect("Failed to obtain Peripherals");
    let mut twatch = twatch::TWatch::new(dp);

    // draw image on black background
    // initialize
    twatch
        .display
        .init(&mut esp32_hal::delay::Delay::new())
        .unwrap();

    dprintln!("\n\nESP32 Started\n\n");

    let raw_image_data: ImageRawLE<Rgb565> =
        ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 8));
    // draw image on black background
    match twatch.display.clear(Rgb565::BLACK) {
        Ok(()) => dprint!("Ok\r\n"),
        Err(_e) => dprint!("KO\r\n"),
    };
    match ferris.draw(&mut twatch.display) {
        Ok(()) => dprint!("Ok\r\n"),
        Err(_e) => dprint!("KO\r\n"),
    };

    loop {
        match display_debug(&mut twatch) {
            Ok(()) => dprint!("Ok !\r\n"),
            Err(_) => dprint!("KO !\r\n"),
        };
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("Panic: {:?}\r\n", info);
    loop {}
}
