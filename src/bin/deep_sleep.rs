#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::time::Duration as CoreDuration;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
};
use log::info;

use embassy_executor::Spawner;

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rtc = Rtc::new(peripherals.LPWR);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized! Staying active for 10s");

    Timer::after(Duration::from_millis(10000)).await;

    info!("Going to sleep now at {}, bye bye", rtc.current_time());
    Timer::after(Duration::from_millis(100)).await;

    rtc.sleep_deep(&[&TimerWakeupSource::new(CoreDuration::from_millis(10000))]);
}

#[global_allocator]
static DUMMY: Dummy = Dummy;

struct Dummy;

unsafe impl core::alloc::GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        extern "C" {
            fn alloc_blocker() -> *mut u8;
        }

        alloc_blocker()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        extern "C" {
            fn dealloc_blocker();
        }

        dealloc_blocker()
    }
}
