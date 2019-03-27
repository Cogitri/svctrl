use crate::configuration::Config;
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;

/*
 * Try to print all directories found inside the path of the ServiceDir
 */

/// Return Option that is either a vector of strings of all directories or None, it returns None if
/// the given path is not a directory.
pub(crate) fn show_dirs(p: &PathBuf) -> Option<Vec<String>> {
    let mut vec = Vec::new();

    if p.is_dir() {
        for entry in fs::read_dir(&p).unwrap() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();

            if path.is_dir() {
                let file = match path.file_name() {
                    Some(e) => e,
                    None => continue,
                };
                vec.push(file.to_str().unwrap().to_string());
            }
        }
    } else {
        return None;
    }

    Some(vec)
}

/// Returns either a vector of strings representing the name of the directories in the active
/// services directory or None if there are no services active
pub(crate) fn show_active_services(c: &Config) -> Option<Vec<String>> {
    match show_dirs(&c.lndir) {
        Some(e) => Some(e),
        None => None,
    }
}
