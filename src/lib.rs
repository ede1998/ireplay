#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub const WEB_TASK_POOL_SIZE: usize = 8;

pub mod wifi;
pub mod server;