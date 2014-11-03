//! Reader and writer for the DEFLATE compression method
//!
//! Both of the streams use an internal buffer. This buffer will be flushed when the object is
//! dropped.
//!
//! ```
//! fn deflate() -> std::io::IoResult<()>
//! {
//!     let mut stdin = std::io::stdin();
//!     let stdout = std::io::stdout();
//!     let mut deflater = flatestream::DeflateWriter::new(stdout, true);
//!
//!     loop
//!     {
//!         let b = try!(stdin.read_u8());
//!         try!(deflater.write_u8(b));
//!     }
//! }
//!
//! println!("Result: {}", deflate());
//! ```

#![feature(unsafe_destructor, phase)]
#![warn(missing_docs)]

extern crate libc;
#[phase(plugin, link)] extern crate log;

pub use reader::InflateReader;
pub use writer::DeflateWriter;

const READ_BUFFER_SIZE : uint = 512;
const REDUCE_MIN_TOTAL_LEN : uint = 2048;
const REDUCE_MAX_AVAIL_LEN : uint = 1024;
const WRITE_FLUSH_MIN_AVAIL : uint = 128;
const WRITE_BUFFER_ADDITIONAL_SIZE : uint = 32;

mod miniz;
mod reader;
mod writer;
