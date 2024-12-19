#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{prelude::*, rng::Rng, timer::timg::TimerGroup};
use ireplay::{server, wifi};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Debug);
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let config = config;
    let peripherals = esp_hal::init(config);
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rng = Rng::new(peripherals.RNG);

    // need SRAM for WIFI but can also use PSRAM for other stuff
    // SRAM must be first to prevent WIFI from choosing PSRAM
    esp_alloc::heap_allocator!(72 * 1024);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    esp_hal_embassy::init(timg1.timer0);

    let stack = wifi::WifiBuilder {
        wifi: peripherals.WIFI,
        radio_clock: peripherals.RADIO_CLK,
        timer: timg0.timer0,
        rng,
    }
    .connect(&spawner)
    .await;

    server::init(&spawner, stack).await;
}
