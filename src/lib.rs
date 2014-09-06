#![feature(unsafe_destructor)]

extern crate libc;

pub use reader::InflateReader;

mod miniz;
pub mod reader;
