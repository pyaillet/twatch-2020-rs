#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]

use core::{self, fmt::Write, panic::PanicInfo};

use embedded_graphics::Drawable;

use watchface::{
    battery::{ChargerState, StateOfCharge},
    time::Time,
    SimpleWatchfaceStyle, Watchface,
};

use esp32_hal::{prelude::*, target};

use twatch::{self, dprint, dprintln, TWatchError};

use heapless::String;

fn display_debug(twatch: &mut twatch::TWatch<'static>) -> Result<(), TWatchError> {
    let charge = twatch
        .pmu
        .is_charging()
        .map_err(|_e| TWatchError::PMUError)?;
    dprintln!("{}\r", charge);

    let percentage = twatch
        .get_battery_percentage()
        .map_err(|_e| TWatchError::PMUError)?;

    let charger_state = match (percentage, charge) {
        (_, true) => ChargerState::Charging,
        (100, false) => ChargerState::Full,
        (_, false) => ChargerState::Discharging,
    };

    let time = twatch
        .rtc
        .get_datetime()
        .map_err(|_| TWatchError::RTCError)?;
    let mut time_str: String<8> = String::new();
    write!(
        time_str,
        "{}",
        format_args!(
            "{:>2}:{:02}:{:02}",
            &time.hours, &time.minutes, &time.seconds
        )
    )
    .unwrap();
    dprintln!("{}\r", time_str);

    let style = SimpleWatchfaceStyle::default();

    let watchface = Watchface::build()
        .with_time(Time {
            hours_local: time.hours,
            minutes_local: time.minutes,
            seconds_local: time.seconds,
        })
        .with_battery(StateOfCharge::from_percentage(percentage))
        .with_charger(charger_state)
        .into_styled(style);

    watchface
        .draw(&mut twatch.display)
        .map_err(|_| TWatchError::DisplayError)
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
        twatch::sleep(10_000_000.us());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("Panic: {:?}\r\n", info);
    loop {}
}
