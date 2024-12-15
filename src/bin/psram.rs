#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(try_with_capacity)]
#![feature(num_midpoint)]
#![feature(alloc_layout_extra)]

use core::alloc::{GlobalAlloc, Layout};

use alloc::vec::Vec;
use esp_backtrace as _;
use esp_hal::prelude::*;
use log::info;

use embassy_executor::Spawner;

extern crate alloc;

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    #[rustfmt::skip]
    let alignments = [
       1, 
       2, 
       4, 
       8, 
       16, 
       32, 
       64
    ];

    for align in alignments {
        info!("Starting binary search");
        let limit = binary_search(|n| allocate_vec(n, align));
        info!("Can allocate at most {limit} bytes aligned to {align}");
    }
}

fn binary_search<F: Fn(usize) -> bool>(check: F) -> usize {
    let mut lower = 0;
    let mut upper = usize::MAX;
    while lower < upper {
        let mid = lower.midpoint(upper);
        // info!("lower = {lower}, upper = {upper}, mid = {mid}");
        if check(mid) {
            lower = mid + 1;
        } else {
            upper = mid - 1;
        }
    }
    assert_eq!(lower, upper);

    if check(lower) {
        lower
    } else {
        lower - 1
    }
}

#[allow(dead_code)]
fn allocate(capacity: usize, align: usize) -> bool {
    let Ok(layout) = Layout::from_size_align(capacity, align) else {
        info!("FAILED at {capacity} bytes");
        return false;
    };

    if check_linked_list_panic(layout) {
        info!("FAILED at {capacity} bytes because of layout calc");
        return false;
    }

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
        true
    }
}

fn check_linked_list_panic(layout: Layout) -> bool {
    let mut size = layout.size();
    const MIN_SIZE: usize = size_of::<usize>() * 2;
    if size < MIN_SIZE {
        size = MIN_SIZE;
    }
    let size = align_up_size(size, 8 /*core::mem::align_of::<Hole>()*/);
    Layout::from_size_align(size, layout.align()).is_err()
}

#[allow(dead_code)]
fn allocate_vec(capacity: usize, align: usize) -> bool {
    info!("Called with {capacity} and {align}");
    fn inner<T: Default>(capacity: usize) -> bool {
        let Ok((layout, _)) = Layout::new::<T>().repeat(capacity) else {
            info!("FAILED at {capacity} bytes");
            return false;
        };
        if check_linked_list_panic(layout) {
            info!("FAILED at {capacity} bytes because of layout calc");
            return false;
        }

        match Vec::try_with_capacity(capacity) {
            Ok(mut v) => {
                assert_eq!(v.capacity(), capacity);
                v.resize_with(capacity, T::default);
                drop(v);
                info!("Allocated {capacity} bytes");
                true
            }
            Err(_) => {
                info!("FAILED at {capacity} bytes");
                false
            }
        }
    }

    #[repr(align(16))]
    #[derive(Default)]
    struct A16([u8; 16]);

    #[repr(align(32))]
    #[derive(Default)]
    struct A32([u8; 32]);

    #[repr(align(64))]
    struct A64([u8; 64]);

    impl Default for A64 {
        fn default() -> Self {
            Self([0; 64])
        }
    }

    match align {
        1 => inner::<u8>(capacity),
        2 => inner::<u16>(capacity),
        4 => inner::<u32>(capacity),
        8 => inner::<u64>(capacity),
        16 => inner::<A16>(capacity),
        32 => inner::<A32>(capacity),
        64 => inner::<A64>(capacity),
        n => panic!("align {n} not implemented"),
    }
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down_size(size: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        size & !(align - 1)
    } else if align == 0 {
        size
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up_size(size: usize, align: usize) -> usize {
    align_down_size(size + align - 1, align)
}
