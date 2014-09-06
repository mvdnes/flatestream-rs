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

    fn mz_deflateInit(pStream: *mut mz_stream_s, level: c_int) -> c_int;
    fn mz_deflate(pStream: *mut mz_stream_s, flush: c_int) -> c_int;
    fn mz_deflateEnd(pStream: *mut mz_stream_s) -> c_int;
}

static MZ_SYNC_FLUSH : c_int = 2;
static MZ_FINISH : c_int = 4;

#[deriving(FromPrimitive)]
enum MzError
{
    StreamEnd = 1,
    NeedDict = 2,
    ErrNo = -1,
    StreamError = -2,
    DataError = -3,
    MemError = -4,
    BufError = -5,
    VersionError = -6,
    ParamError = -10000,
}

pub struct InflateReader<R>
{
    miniz_data: mz_stream_s,
    inner: R,
    buffer: Vec<u8>,
    buffer_pos: uint,
    eof: bool,
}

fn miniz_code_to_result(code: c_int) -> io::IoResult<()>
{
    if code == 0 { return Ok(()) }
    let result : Option<MzError> = FromPrimitive::from_u64(code as u64);
    match result
    {
        Some(e) => match e
        {
            StreamEnd => Err(io::standard_error(io::EndOfFile)),
            NeedDict | ErrNo | StreamError | DataError | VersionError => Err(io::standard_error(io::MismatchedFileTypeForOperation)),
            MemError => fail!("Miniz memory error occured"),
            BufError => fail!("Miniz buffer error occured"),
            ParamError => fail!("Miniz parameter error occured"),
        },
        None => fail!("Miniz produced an unknown error"),
    }
}

impl<R: Reader+Send> InflateReader<R>
{
    pub fn new(inner: R) -> io::IoResult<InflateReader<R>>
    {
        let mut miniz_data = mz_stream_s::empty();
        try!(miniz_code_to_result(unsafe { mz_inflateInit(&mut miniz_data) }));
        Ok(InflateReader
        {
            miniz_data: miniz_data,
            inner: inner,
            buffer: Vec::new(),
            buffer_pos: 0,
            eof: false,
        })
    }

    fn read_inner(&mut self) -> io::IoResult<uint>
    {
        if self.buffer_len() >= 512 { return Ok(0) }
        self.reduce_buffer_if_needed();

        let mut temp_buffer = [0, ..512];
        match self.inner.read(&mut temp_buffer)
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
        if start < len
        {
            self.buffer.mut_slice(start, len).as_mut_ptr()
        }
        else
        {
            ptr::mut_null()
        }
    }

    fn reduce_buffer_if_needed(&mut self)
    {
        if self.buffer.len() < 2048 || self.buffer_len() > 1024
        {
            return
        }

        self.buffer = Vec::from_slice(self.buffer.slice(self.buffer_pos, self.buffer.len()));
        self.buffer_pos = 0;
    }
}

#[unsafe_destructor]
impl<R: Reader+Send> Drop for InflateReader<R>
{
    fn drop(&mut self)
    {
        unsafe { mz_inflateEnd(&mut self.miniz_data); }
    }
}

impl<R: Reader+Send> Reader for InflateReader<R>
{
    fn read(&mut self, buf: &mut [u8]) -> io::IoResult<uint>
    {
        if self.eof { return Err(io::standard_error(io::EndOfFile)) }

        match self.read_inner()
        {
            Ok(_) => {},
            Err(ref e) if e.kind == io::EndOfFile => {},
            Err(e) => { return Err(e) },
        }

        self.miniz_data.avail_out = buf.len() as c_uint;
        self.miniz_data.next_out = buf.as_mut_ptr();
        self.miniz_data.avail_in = self.buffer_len() as c_uint;
        self.miniz_data.next_in = self.buffer_ptr();

        let prev_out = self.miniz_data.total_out;
        let prev_in = self.miniz_data.total_in;

        let code = unsafe { mz_inflate(&mut self.miniz_data, MZ_SYNC_FLUSH) };

        let read = (self.miniz_data.total_out - prev_out) as uint;
        self.buffer_pos += (self.miniz_data.total_in - prev_in) as uint;

        match miniz_code_to_result(code)
        {
            Ok(_) => Ok(read),
            Err(ref e) if e.kind == io::EndOfFile => { self.eof = true; Ok(read) },
            Err(e) => Err(e),
        }
    }
}
