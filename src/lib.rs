#![feature(unsafe_destructor, phase)]

extern crate libc;
#[phase(plugin, link)] extern crate log;

pub use reader::InflateReader;
pub use writer::DeflateWriter;

static READ_BUFFER_SIZE : uint = 512;
static REDUCE_MIN_TOTAL_LEN : uint = 2048;
static REDUCE_MAX_AVAIL_LEN : uint = 1024;
static WRITE_FLUSH_MIN_AVAIL : uint = 128;
static WRITE_BUFFER_ADDITIONAL_SIZE : uint = 32;

mod miniz;
pub mod reader;
pub mod writer;
