use std::fs::File;
use std::io::{self, Read, Write};

/// Read file `filename` into a `Vec<u8>`.
pub fn read_file(filename: &str) -> Result<Vec<u8>, io::Error> {
    let mut f = try!(File::open(filename));
    let mut buf: Vec<u8> = vec![];
    try!(f.read_to_end(&mut buf));

    Ok(buf)
}

/// Writes `data` to `filename`.
pub fn write_file(filename: &str, data: &[u8]) -> Result<(), io::Error> {
    let mut file = try!(File::create(filename));
    try!(file.write_all(data));
    Ok(())
}
