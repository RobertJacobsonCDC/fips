/*!
This module is enabled with the "aspr_archive" feature and provides routines to read ASPR data from zipped archives.
*/

use std::{
    path::PathBuf,
    sync::RwLock,
    io::BufRead
};
use once_cell::sync::Lazy;

use crate::states::USState;
use super::errors::ASPRError;


// Path to the ASPR data directory
const DEFAULT_ASPR_DATA_PATH: &str = "/Users/rjacobson/Code/CDC/data/ASPR_Synthetic_Population";
static ASPR_DATA_PATH: Lazy<RwLock<PathBuf>> = Lazy::new(|| RwLock::new(PathBuf::from(DEFAULT_ASPR_DATA_PATH)));


/// Setter for the ASPR data directory path.
pub fn set_aspr_data_path(path: PathBuf) {
    *ASPR_DATA_PATH.write().unwrap() = path;
}

/// Setter for the ASPR data directory path.
pub fn get_aspr_data_path() -> PathBuf {
    ASPR_DATA_PATH.read().unwrap().clone()
}

/// Returns an iterator over all the files in the ASPR "all_states" data directory.
pub fn all_states_iter_files()
    -> Result<std::vec::IntoIter<PathBuf>, ASPRError>
{
    let mut path = get_aspr_data_path();
    path.push("all_states");
    let mut files = vec![];
    let entries = path.read_dir().map_err(|e| ASPRError::Io(e) )?;

    for entry in entries {
        let entry = entry.map_err(|e| ASPRError::Io(e) )?;
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    Ok(files.into_iter())
}

pub fn state_row_iter(state: USState) -> Result<(), std::io::Error> {
    let file_name = format!("{}.csv", state.to_string().to_lowercase());
    let mut path  = get_aspr_data_path();

    path.push("all_states");
    path.push(file_name);

    let file      = std::fs::File::open(path)?;
    let line_iter = std::io::BufReader::new(file).lines();

    for line in line_iter {
        let line: String = line?;
        line.split(',');
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_states_iter_files() {
        let files = all_states_iter_files();
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }
    }

    #[test]
    fn test_state_row_iter() {
        let state     = USState::AL;
        let file_name = format!("{}.csv"     , state.to_string().to_lowercase());
        let mut path  = get_aspr_data_path();

        path.push("all_states");
        path.push(file_name);

        let file          = std::fs::File::open(path).unwrap();
        let mut line_iter = std::io::BufReader::new(file).lines();

        // Skip the header row
        line_iter.next();

        for (idx, line) in line_iter.enumerate() {
            if idx == 10 { break; }
            let line          = line.unwrap();
            let mut part_iter = line.split(',');
            let age           = part_iter.next().unwrap().parse::<u8>().unwrap();

            let home_id_str   = part_iter.next().unwrap().trim();
            let school_id_str = part_iter.next().unwrap().trim();
            let work_id_str   = part_iter.next().unwrap().trim();
        }
    }
}
