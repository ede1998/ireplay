#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Output},
    prelude::*,
};
use heapless::Vec;
use log::info;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};

extern crate alloc;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    // TODO: Spawn some tasks
    let _ = spawner;
    let mut rx = Input::new(peripherals.GPIO25, esp_hal::gpio::Pull::Up);
    let _tx = Output::new(peripherals.GPIO17, esp_hal::gpio::Level::Low);

    loop {
        // g25
        // g17
        let mut signal: Vec<u8, 1_000> = heapless::Vec::new();
        // Min. pulse duration = 500us -> double it to protect against aliasing
        const TICK_RATE: Duration = Duration::from_micros(250);
        let mut ticker = Ticker::every(TICK_RATE);
        rx.wait_for_falling_edge().await;
        ticker.reset();
        for _ in 0..signal.capacity() {
            let value = if rx.is_high() { 1 } else { 0 };
            let _: Result<(), _> = signal.push(value);
            ticker.next().await;
        }

        assert_eq!(signal.last(), Some(&1));
        // let last_sample = signal.iter().rposition(|&v| v == 0).unwrap_or_default();
        // let last_sample = (last + 10).min(signal.len());
        const SIGNAL_DURATION: Duration = Duration::from_millis(60);
        let last_sample = (SIGNAL_DURATION.as_micros() / TICK_RATE.as_micros())
            .try_into()
            .unwrap();

        info!("{}", SignalPrinter(&signal[..last_sample]));
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}

struct SignalPrinter<'a>(&'a [u8]);

impl core::fmt::Display for SignalPrinter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for item in self.0 {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}
