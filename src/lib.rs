#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub const WEB_TASK_POOL_SIZE: usize = 8;

pub mod server;
pub mod wifi;

extern crate alloc;
