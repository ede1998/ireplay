#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub const WEB_TASK_POOL_SIZE: usize = 8;

pub mod server;
pub mod wifi;

mod extractor;
mod multi_core;

pub use multi_core::start_2nd_core;

extern crate alloc;
