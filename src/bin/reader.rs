#![feature(phase)]

extern crate flatestream;
#[phase(plugin, link)] extern crate log;

use std::{os, io};

fn main()
{
    let args = os::args();

    let path = Path::new(args.as_slice()[1].as_slice());
    let file = io::fs::File::open(&path);
    let mut deflated = flatestream::InflateReader::new(file, true).unwrap();

    let mut inflated = io::stdout();

    let mut buffer = [0u8, ..4096];

    loop
    {
        match deflated.read(&mut buffer)
        {
            Ok(n) =>
            {
                match inflated.write(buffer.slice(0, n))
                {
                    Err(e) => { error!("Read error: {}", e); break },
                    _ => {},
                }
            },
            Err(ref e) if e.kind == io::EndOfFile => { break },
            Err(e) => { error!("Error: {}", e); break },
        }
    }
}
