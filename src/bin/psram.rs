#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::alloc::{GlobalAlloc, Layout};

use esp_backtrace as _;
use esp_hal::prelude::*;
use log::info;


extern crate alloc;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    esp_println::logger::init_logger_from_env();

    allocate(1048572, 4);

    loop {}
}

fn allocate(capacity: usize, align: usize) -> bool {
    let Ok(layout) = Layout::from_size_align(capacity, align) else {
        info!("FAILED at {capacity} bytes");
        return false;
    };

    let ptr = unsafe { esp_alloc::HEAP.alloc(layout) };
    if ptr.is_null() {
        info!("FAILED at {capacity} bytes");
        false
    } else {
        unsafe {
            core::ptr::write_bytes(ptr, 123, capacity);
        }
        info!("Allocated {capacity} bytes");
        unsafe {
            esp_alloc::HEAP.dealloc(ptr, layout);
        }
        info!("Deallocated {capacity} bytes");
        true
    }
}