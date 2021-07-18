#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]

use core::{self, fmt::Write, panic::PanicInfo};
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use esp32_hal::{prelude::*, target};

use twatch::{self, dprint, dprintln, TWatchError};

use heapless::String;

fn display_debug(twatch: &mut twatch::TWatch<'static>) -> Result<(), TWatchError> {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let charge = if twatch
        .pmu
        .is_charging()
        .map_err(|_e| TWatchError::PMUError)?
    {
        "Charging... "
    } else {
        "Not charging"
    };
    let battery = if twatch
        .pmu
        .is_battery_connect()
        .map_err(|_e| TWatchError::PMUError)?
    {
        "Battery connected    "
    } else {
        "Battery not connected"
    };
    let percentage = twatch
        .get_battery_percentage()
        .map_err(|_e| TWatchError::PMUError)?;

    let mut percentage_str: String<5> = String::new();
    write!(percentage_str, "{} %", format_args!("{}", percentage)).unwrap();

    let time = twatch
        .rtc
        .get_datetime()
        .map_err(|_| TWatchError::RTCError)?;
    let mut time_str: String<5> = String::new();
    write!(
        time_str,
        "{}",
        format_args!("{}:{}", &time.hours, &time.minutes)
    )
    .unwrap();

    Text::new(charge, Point::new(40, 75), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    Text::new(battery, Point::new(40, 85), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    Text::new(percentage_str.as_str(), Point::new(40, 95), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    Text::new(time_str.as_str(), Point::new(40, 115), style)
        .draw(&mut twatch.display)
        .map_err(|_e| TWatchError::DisplayError)?;
    twatch::sleep(1_000_000_u32.us());
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
