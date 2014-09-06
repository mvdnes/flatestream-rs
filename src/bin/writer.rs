#![feature(phase)]

extern crate flatestream;
#[phase(plugin, link)] extern crate log;

use std::{os, io};

fn main()
{
    let args = os::args();

    let path = Path::new(args.as_slice()[1].as_slice());
    let mut deflated = io::fs::File::open(&path);

    let stdout = io::stdout();
    let mut inflated = flatestream::DeflateWriter::new(stdout).unwrap();

    let mut buffer = [0u8, ..4096];

    loop
    {
        match deflated.read(&mut buffer)
        {
            Ok(n) =>
            {
                match inflated.write(buffer.slice(0, n))
                {
                    Err(e) => { error!("Write error: {}", e); break },
                    _ => {},
                }
            },
            Err(ref e) if e.kind == io::EndOfFile => { break },
            Err(e) => { error!("Error: {}", e); break },
        }
    }
}
