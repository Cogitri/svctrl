use std::fs;
use std::path::Path;
use std::io;

// Struct representing a directory where services are kept
pub struct ServiceDir {
    path: Path
}

impl ServiceDir {

    /*
     * Try to print all directories found inside the path of the ServiceDir
     */
    pub fn show_services(&self) -> io::Result<()> {
        if self.path.is_dir() {
            for entry in fs::read_dir(&self.path)? {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        eprintln!("WARN: {} when reading {}", err, self.path.display());
                        continue
                    }
                };

                let path = entry.path();

                if path.is_dir() {
                    let file = match path.file_name() {
                        Some(e) => e,
                        None => {
                            eprintln!("WARN: ServiceDir '{}' returned no file_name.", self.path.display());
                            continue
                        }
                    };
                    println!("{}", file.to_str().unwrap());
                }
            }
        }
        Ok(())
    }

}

