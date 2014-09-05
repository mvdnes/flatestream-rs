#![feature(unsafe_destructor)]

extern crate libc;

use std::ptr;
use libc::{c_void, c_int, c_ulong, c_uint};
use std::io;

#[repr(C)]
struct mz_stream_s
{
    next_in: *mut u8,
    avail_in: c_uint,
    total_in: c_ulong,

    next_out: *mut u8,
    avail_out: c_uint,
    total_out: c_ulong,

    msg: *mut u8,
    state: *mut c_void,

    zalloc: *const c_void,
    zfree: *const c_void,
    opaque: *const c_void,

    data_type: c_int,
    adler: c_ulong,
    reserved: c_ulong,
}

impl mz_stream_s
{
    fn empty() -> mz_stream_s
    {
        mz_stream_s
        {
            next_in: ptr::mut_null(), avail_in: 0, total_in: 0,
            next_out: ptr::mut_null(), avail_out: 0, total_out: 0,
            msg: ptr::mut_null(), state: ptr::mut_null(),
            zalloc: ptr::null(), zfree: ptr::null(), opaque: ptr::null(),
            data_type: 0, adler: 0, reserved: 0,
        }
    }
}

#[link(name = "miniz", kind = "static")]
extern "C"
{
    fn mz_inflateInit(pStream: *mut mz_stream_s) -> c_int;
    fn mz_inflate(pStream: *mut mz_stream_s, flush: c_int) -> c_int;
    fn mz_inflateEnd(pStream: *mut mz_stream_s) -> c_int;
}

static STREAM_END : c_int = 1;
static SYNC_FLUSH : c_int = 2;

pub struct InflateReader<'a>
{
    miniz_data: mz_stream_s,
    inner: &'a mut Reader+'a,
    buffer: Vec<u8>,
    buffer_pos: uint,
    eof: bool,
}

impl<'a> InflateReader<'a>
{
    pub fn new(inner: &'a mut Reader) -> InflateReader<'a>
    {
        let mut miniz_data = mz_stream_s::empty();
        unsafe { mz_inflateInit(&mut miniz_data); }
        InflateReader
        {
            miniz_data: miniz_data,
            inner: inner,
            buffer: Vec::new(),
            buffer_pos: 0,
            eof: false,
        }
    }

    fn read_inner(&mut self) -> io::IoResult<uint>
    {
        if self.buffer.len() >= 64 { return Ok(0) }
        let mut temp_buffer = [0, ..64];
        match self.inner.read_at_least(1, &mut temp_buffer)
        {
            Err(e) => Err(e),
            Ok(n) =>
            {
                for i in range(0, n)
                {
                    self.buffer.push(temp_buffer[i]);
                }
                Ok(n)
            }
        }
    }

    fn buffer_len(&self) -> uint
    {
        self.buffer.len() - self.buffer_pos
    }

    fn buffer_ptr(&mut self) -> *mut u8
    {
        let len = self.buffer.len();
        let start = self.buffer_pos;
        self.buffer.mut_slice(start, len).as_mut_ptr()
    }
}

#[unsafe_destructor]
impl<'a> Drop for InflateReader<'a>
{
    fn drop(&mut self)
    {
        unsafe { mz_inflateEnd(&mut self.miniz_data); }
    }
}

impl<'a> Reader for InflateReader<'a>
{
    fn read(&mut self, buf: &mut [u8]) -> io::IoResult<uint>
    {
        if self.eof { return Err(io::standard_error(io::EndOfFile)) }
        self.read_inner();
        self.miniz_data.avail_out = buf.len() as c_uint;
        self.miniz_data.next_out = buf.as_mut_ptr();
        self.miniz_data.avail_in = self.buffer_len() as c_uint;
        self.miniz_data.next_in = self.buffer_ptr();

        let prev_out = self.miniz_data.total_out;
        let code = unsafe { mz_inflate(&mut self.miniz_data, SYNC_FLUSH) };
        let read = (self.miniz_data.total_out - prev_out) as uint;

        match code
        {
            0 => Ok(read),
            1 => { self.eof = true; Ok(read) },
            _ => Err(io::standard_error(io::MismatchedFileTypeForOperation)),
        }
    }
}
