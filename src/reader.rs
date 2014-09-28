use std::ptr;
use libc::c_uint;
use std::io;
use miniz;

/// Reader that automatically inflates data from the contained reader
pub struct InflateReader<R>
{
    miniz_data: miniz::mz_stream_s,
    inner: R,
    buffer: Vec<u8>,
    buffer_pos: uint,
    eof: bool,
}

impl<R: Reader+Send> InflateReader<R>
{
    /// Construct a new InflateReader, which inflates data from the inner Reader.
    pub fn new(inner: R, zlib_header: bool) -> io::IoResult<InflateReader<R>>
    {
        let mut miniz_data = miniz::mz_stream_s::empty();
        let flag = if zlib_header { 1 } else { -1 } * miniz::MZ_DEFAULT_WINDOW_BITS;
        try!(miniz::code_to_result(unsafe { miniz::mz_inflateInit2(&mut miniz_data, flag) }));
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
        if self.buffer_len() >= ::READ_BUFFER_SIZE { return Ok(0) }
        self.reduce_buffer();

        let mut temp_buffer = [0, ..::READ_BUFFER_SIZE];
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

    fn buffer_ptr(&mut self) -> *const u8
    {
        let len = self.buffer.len();
        let start = self.buffer_pos;
        if start < len
        {
            self.buffer.slice_mut(start, len).as_ptr()
        }
        else
        {
            ptr::null()
        }
    }

    fn reduce_buffer(&mut self)
    {
        if self.buffer.len() < ::REDUCE_MIN_TOTAL_LEN || self.buffer_len() > ::REDUCE_MAX_AVAIL_LEN
        {
            return
        }

        self.buffer = self.buffer.slice(self.buffer_pos, self.buffer.len()).to_vec();
        self.buffer_pos = 0;
    }
}

#[unsafe_destructor]
impl<R: Reader+Send> Drop for InflateReader<R>
{
    fn drop(&mut self)
    {
        unsafe { miniz::mz_inflateEnd(&mut self.miniz_data); }
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

        let code = unsafe { miniz::mz_inflate(&mut self.miniz_data, miniz::MZ_SYNC_FLUSH) };

        let read = (self.miniz_data.total_out - prev_out) as uint;
        self.buffer_pos += (self.miniz_data.total_in - prev_in) as uint;

        match miniz::code_to_result(code)
        {
            Ok(_) => Ok(read),
            Err(ref e) if e.kind == io::EndOfFile => { self.eof = true; Ok(read) },
            Err(e) => Err(e),
        }
    }
}
