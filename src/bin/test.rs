extern crate flatestream;

use std::{os, io};

fn main()
{
    let args = os::args();

    let path = Path::new(args.as_slice()[1].as_slice());
    let mut file = io::fs::File::open(&path);

    let mut fstream = flatestream::InflateReader::new(&mut file);
    match fstream.read_to_end()
    {
        Ok(vec) =>
        {
            for u in vec.iter()
            {
                print!("{}", *u as char);
            }
        }
        Err(e) => { println!("Error: {}", e); },
    }
}
