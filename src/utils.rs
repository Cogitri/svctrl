extern crate unix_named_pipe;

use crate::errors::Error;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

// Function to write to a fifo
pub fn write_to_fifo(p: PathBuf, a: &str) -> Result<(), Error> {
    // Try to open the fifo
    let mut fifo = match unix_named_pipe::open_write(&p) {
        Ok(p) => p,
        Err(e) => return Err(Error::Open(p, e)),
    };

    match write!(fifo, "{}", a) {
        Ok(_) => Ok(()),
        Err(e) => return Err(Error::Write(p, e)),
    }
}

// Open a file for reading and read it, return an error if unable to
// write to it or open it.
pub fn read_file(p: &PathBuf) -> Result<String, Error> {
    return match read_to_string(&p) {
        Ok(s) => Ok(s),
        Err(e) => return Err(Error::Read(p.clone(), e)),
    };
}
