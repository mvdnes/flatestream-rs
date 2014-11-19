use std::ptr;
use libc::{c_void, c_int, c_ulong, c_uint};
use std::io;

#[repr(C)]
pub struct mz_stream_s
{
    pub next_in: *const u8,
    pub avail_in: c_uint,
    pub total_in: c_ulong,

    pub next_out: *mut u8,
    pub avail_out: c_uint,
    pub total_out: c_ulong,

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
    pub fn empty() -> mz_stream_s
    {
        mz_stream_s
        {
            next_in: ptr::null(), avail_in: 0, total_in: 0,
            next_out: ptr::null_mut(), avail_out: 0, total_out: 0,
            msg: ptr::null_mut(), state: ptr::null_mut(),
            zalloc: ptr::null(), zfree: ptr::null(), opaque: ptr::null(),
            data_type: 0, adler: 0, reserved: 0,
        }
    }
}

#[link(name = "miniz", kind = "static")]
extern "C"
{
    pub fn mz_inflateInit2(pStream: *mut mz_stream_s, window_bits: c_int) -> c_int;
    pub fn mz_inflate(pStream: *mut mz_stream_s, flush: c_int) -> c_int;
    pub fn mz_inflateEnd(pStream: *mut mz_stream_s) -> c_int;

    pub fn mz_deflateInit2(pStream: *mut mz_stream_s, level: c_int, method: c_int, window_bits: c_int, mem_level: c_int, strategy: c_int) -> c_int;
    pub fn mz_deflate(pStream: *mut mz_stream_s, flush: c_int) -> c_int;
    pub fn mz_deflateEnd(pStream: *mut mz_stream_s) -> c_int;
}

pub static MZ_DEFAULT_WINDOW_BITS : c_int = 15;
pub static MZ_NO_FLUSH : c_int = 0;
pub static MZ_SYNC_FLUSH : c_int = 2;
pub static MZ_FINISH : c_int = 4;

#[deriving(FromPrimitive)]
pub enum MzError
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

pub fn code_to_result(code: c_int) -> io::IoResult<()>
{
    if code == 0 { return Ok(()) }
    let result : Option<MzError> = FromPrimitive::from_u64(code as u64);
    match result
    {
        Some(e) => match e
        {
            MzError::StreamEnd => Err(io::standard_error(io::EndOfFile)),
            MzError::NeedDict | MzError::ErrNo | MzError::StreamError | MzError::DataError | MzError::VersionError => Err(io::standard_error(io::MismatchedFileTypeForOperation)),
            MzError::MemError => panic!("Miniz memory error occured"),
            MzError::BufError => panic!("Miniz buffer error occured"),
            MzError::ParamError => panic!("Miniz parameter error occured"),
        },
        None => panic!("Miniz produced an unknown error"),
    }
}
