use crate::errors::Error;
use std::fmt::Write as FmtWrite;
use std::fs::read_to_string;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::Command;

// Function to write to a fifo
pub fn write_to_fifo(p: PathBuf, a: String) -> Result<String, Error> {
    // Try to open the fifo
    let mut fifo = match OpenOptions::new().read(true).write(true).open(&p) {
        Ok(p) => p,
        Err(e) => return Err(Error::Open(p.into_os_string().into_string().unwrap(), e)),
    };

    // NOTE: find a way to do it in pure rust
    let child = Command::new("/bin/head")
        .arg("-c")
        .arg("1")
        .arg(&p)
        .spawn()
        .expect("Failed to attach to fifo!");

    match write!(fifo, "{}", a) {
        Ok(_) => (),
        Err(e) => return Err(Error::Write(p.into_os_string().into_string().unwrap(), e)),
    }

    let stdout: Vec<u8> = match child.wait_with_output() {
        Ok(f) => f.stdout,
        Err(e) => return Err(Error::Read(p.into_os_string().into_string().unwrap(), e)),
    };

    let mut out = String::new();

    for n in stdout {
        let _ = write!(&mut out, "{}", n);
    }

    Ok(out)
}

// Open a file for reading and write to it, return an error if unable to
// write to it or open it.
pub fn read_file(p: PathBuf) -> Result<String, Error> {
    return match read_to_string(&p) {
        Ok(s) => Ok(s),
        Err(e) => return Err(Error::Read(p.into_os_string().into_string().unwrap(), e)),
    };
}
