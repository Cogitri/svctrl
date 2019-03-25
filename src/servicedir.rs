use std::fs;
use std::path::PathBuf;
use std::vec::Vec;

/*
 * Try to print all directories found inside the path of the ServiceDir
 */
pub(crate) fn show_services(p: PathBuf) -> Option<Vec<String>> {
    let mut vec = Vec::new();

    if p.is_dir() {
        for entry in fs::read_dir(&p).unwrap() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!("WARN: {} when reading {}", err, p.display());
                    continue
                }
            };

            let path = entry.path();

            if path.is_dir() {
                let file = match path.file_name() {
                    Some(e) => e,
                    None => {
                        eprintln!("WARN: ServiceDir '{}' returned no file_name.", p.display());
                        continue
                    }
                };
                vec.push(file.to_str().unwrap().to_string());
            }
        }
    } else {
        eprintln!("{} is not a valid path", p.display());
        return None
    }

    Some(vec)
}



