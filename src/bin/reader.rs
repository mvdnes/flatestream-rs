extern crate flatestream;

use std::{os, io};

fn main()
{
    let args = os::args();

    let path = Path::new(args.as_slice()[1].as_slice());
    let file = io::fs::File::open(&path);
    let mut fstream = flatestream::InflateReader::new(file).unwrap();

    loop
    {
        match fstream.read_byte()
        {
            Ok(u) =>
            {
                print!("{}", u as char);
            }
            Err(ref e) if e.kind == io::EndOfFile => { break },
            Err(e) => { println!("Error: {}", e); break },
        }
    }
}
