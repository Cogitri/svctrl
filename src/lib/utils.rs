use crate::errors::Error;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

/// Writes to a fifo and returns and raises an error if not possible
///
/// # Arguments
///
/// * `p` - `PathBuf` to where the fifo that should be written is
/// * `a` - A slice string that should be written to the fifo
///
/// # Example
///
/// ```
/// let p = PathBuf::new("/run/fifo");
/// let s = "test";
///
/// write_to_fifo(&p, s)?
/// ```
pub fn write_to_fifo(p: PathBuf, a: &str) -> Result<(), Error> {
    // Try to open the fifo
    let mut fifo = match unix_named_pipe::open_write(&p) {
        Ok(p) => p,
        Err(e) => return Err(Error::Open(p, e)),
    };

    match write!(fifo, "{}", a) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Write(p, e)),
    }
}

/// Reads a file to a string and returns it or raises an error
///
/// # Arguments
///
/// * `p` - `PathBuf` to file that should be read
///
/// # Example
///
/// ```
/// let file = PathBuf::new("foo.txt");
///
/// match read_file(&file) {
///     Ok(r) => println!("{}", r),
///     Err(e) => Err(e),
/// }
/// ```
pub fn read_file(p: &PathBuf) -> Result<String, Error> {
    match read_to_string(&p) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::Read(p.clone(), e)),
    }
}
