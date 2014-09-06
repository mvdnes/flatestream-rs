use libc::{c_int, c_uint};
use std::ptr;
use std::io;
use miniz;

/// Writer that automatically deflates output and sends it to the contained writer
pub struct DeflateWriter<W>
{
    miniz_data: miniz::mz_stream_s,
    inner: W,
    buffer: Vec<u8>,
    buffer_pos: uint,
}

static FLATE_LEVEL : c_int = 6;

impl<W: Writer+Send> DeflateWriter<W>
{
    /// Constructs a new DeflateWriter, which writes deflated output to the inner writer.
    pub fn new(inner: W) -> io::IoResult<DeflateWriter<W>>
    {
        let mut miniz_data = miniz::mz_stream_s::empty();
        try!(miniz::code_to_result(unsafe { miniz::mz_deflateInit(&mut miniz_data, FLATE_LEVEL) }));
        Ok(DeflateWriter
        {
            miniz_data: miniz_data,
            inner: inner,
            buffer: Vec::new(),
            buffer_pos: 0,
        })
    }

    fn buffer_len(&self) -> uint
    {
        self.buffer.len() - self.buffer_pos
    }

    fn buffer_ptr(&self) -> *const u8
    {
        if self.buffer_pos < self.buffer.len()
        {
            self.buffer.slice(self.buffer_pos, self.buffer.len()).as_ptr()
        }
        else
        {
            ptr::null()
        }
    }

    fn flush(&mut self) -> io::IoResult<()>
    {
        if self.buffer_len() < ::WRITE_FLUSH_MIN_AVAIL { return Ok(()) }
        self.flush_generic(miniz::MZ_NO_FLUSH)
    }

    fn flush_generic(&mut self, flag: c_int) -> io::IoResult<()>
    {
        let mut additional_size = ::WRITE_BUFFER_ADDITIONAL_SIZE;
        loop
        {
            let mut out_buffer = Vec::with_capacity(self.buffer_len() + additional_size);

            let prev_in = self.miniz_data.total_in as uint;
            let prev_out = self.miniz_data.total_out as uint;

            self.miniz_data.next_in = self.buffer_ptr();
            self.miniz_data.avail_in = self.buffer_len() as c_uint;
            self.miniz_data.next_out = out_buffer.as_mut_ptr();
            self.miniz_data.avail_out = out_buffer.capacity() as c_uint;

            match miniz::code_to_result(unsafe { miniz::mz_deflate(&mut self.miniz_data, flag) })
            {
                Err(ref e) if e.kind == io::EndOfFile => {},
                Ok(()) => {},
                Err(e) => return Err(e),
            }

            let read = self.miniz_data.total_in as uint - prev_in;
            let written = self.miniz_data.total_out as uint - prev_out;

            self.buffer_pos += read;
            unsafe { out_buffer.set_len(written); }

            try!(self.inner.write(out_buffer.as_slice()));

            if self.miniz_data.avail_out != 0 { break; }
            additional_size = additional_size + additional_size / 2;
        }
        self.reduce_buffer();
        Ok(())
    }

    fn reduce_buffer(&mut self)
    {
        if self.buffer.len() < ::REDUCE_MIN_TOTAL_LEN || self.buffer_len() > ::REDUCE_MAX_AVAIL_LEN
        {
            return
        }

        self.buffer = Vec::from_slice(self.buffer.slice(self.buffer_pos, self.buffer.len()));
        self.buffer_pos = 0;
    }
}

#[unsafe_destructor]
impl<W: Writer+Send> Drop for DeflateWriter<W>
{
    fn drop(&mut self)
    {
        match self.flush_generic(miniz::MZ_FINISH)
        {
            Err(e) => error!("Error: {}", e),
            _ => {},
        }
        unsafe { miniz::mz_deflateEnd(&mut self.miniz_data); }
    }
}

impl<W: Writer+Send> Writer for DeflateWriter<W>
{
    fn write(&mut self, buf: &[u8]) -> io::IoResult<()>
    {
        self.buffer.push_all(buf);
        try!(self.flush());
        Ok(())
    }
}
