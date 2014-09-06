#![feature(unsafe_destructor, phase)]

extern crate libc;
#[phase(plugin, link)] extern crate log;

pub use reader::InflateReader;
pub use writer::DeflateWriter;

mod miniz;
pub mod reader;
pub mod writer;
