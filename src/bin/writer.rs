#![feature(phase)]

#[phase(plugin, link)] extern crate log;

extern crate flatestream;

use std::{os, io};

fn main()
{
    let args = os::args();

    let path = Path::new(args.as_slice()[1].as_slice());
    let mut file = io::fs::File::open(&path);

    let out = io::stdout();

    let mut fstream = flatestream::DeflateWriter::new(out).unwrap();

    loop
    {
        match file.read_byte()
        {
            Ok(u) =>
            {
                match fstream.write(&[u])
                {
                    Ok(()) => {},
                    Err(e) => { error!("Write error!: {}", e); break },
                }
            },
            Err(ref e) if e.kind == io::EndOfFile => { break },
            Err(e) => { error!("Error: {}", e); break },
        }
    }
}
