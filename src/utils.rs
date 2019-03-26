use crate::errors::Error;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

// Function to write to a fifo
pub fn write_to_fifo(p: PathBuf, a: String) -> Result<(), Error> {
    // Try to open the fifo
    let mut fifo = match File::open(&p) {
        Ok(p) => p,
        Err(e) => return Err(Error::Open(p.into_os_string().into_string().unwrap(), e)),
    };

    match write!(fifo, "b\"{}\"", a) {
        Ok(_) => Ok(()),
        Err(e) => return Err(Error::Write(p.into_os_string().into_string().unwrap(), e)),
    }
}

// Open a file for reading and write to it, return an error if unable to
// write to it or open it.
pub fn read_file(p: PathBuf) -> Result<String, Error> {
    return match read_to_string(&p) {
        Ok(s) => Ok(s),
        Err(e) => return Err(Error::Read(p.into_os_string().into_string().unwrap(), e)),
    };
}
