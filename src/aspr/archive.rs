/*!
This module is enabled with the "aspr_archive" feature and provides routines to read ASPR synthetic population data.
These routines can also read from zipped archives.
*/

use std::{
    path::PathBuf,
    sync::RwLock,
    io::BufRead
};
use once_cell::sync::Lazy;
use crate::{
    aspr::{
        parser::{parse_fips_home_id, parse_fips_school_id, parse_fips_workplace_id},
        ASPRPersonRecord
    },
    states::USState
};
use super::errors::ASPRError;

// Directory structure of the ASPR data
const ALL_STATES_DIR         : &str = "all_states";
const CBSA_ALL_DIR           : &str = "cbsa_all_work_school_household";
const CBSA_ONLY_RESIDENTS_DIR: &str = "cbsa_only_residents";
// Either of the next two can be affixed to either of the two above directories
const NON_CBSA_RESIDENTS_DIR : &str = "non_CBSA_residents";
const MULTI_STATE_DIR        : &str = "Multi-state";


// Path to the ASPR data directory
const DEFAULT_ASPR_DATA_PATH: &str = "../CDC/data/ASPR_Synthetic_Population";
static ASPR_DATA_PATH: Lazy<RwLock<PathBuf>> = Lazy::new(|| RwLock::new(PathBuf::from(DEFAULT_ASPR_DATA_PATH)));


/// Setter for the ASPR data directory path.
pub fn set_aspr_data_path(path: PathBuf) {
    *ASPR_DATA_PATH.write().unwrap() = path;
}

/// Getter for the ASPR data directory path.
pub fn get_aspr_data_path() -> PathBuf {
    ASPR_DATA_PATH.read().unwrap().clone()
}

// ToDo: Should we just return a vector? We construct it anyway.
/// Returns an iterator over all the files in the ASPR "all_states" data directory.
pub fn iter_all_states_files()
    -> Result<std::vec::IntoIter<PathBuf>, ASPRError>
{
    let mut path = get_aspr_data_path();
    path.push(ALL_STATES_DIR);
    
    let mut files = vec![];
    let entries   = path.read_dir().map_err(|e| ASPRError::Io(e) )?;

    for entry in entries {
        let entry = entry.map_err(|e| ASPRError::Io(e) )?;
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    Ok(files.into_iter())
}

/// Returns an iterator over all the files in the ASPR "cbsa_all_work_school_household" data directory. In practice,
/// there are three use cases for subdirectory: state, multi-state, and "non_CBSA_residents".
/// 
/// For a specific state, call: `iter_cbsa_all_files(state.as_str())` <br>
/// For multi-state, call: `iter_cbsa_all_files(MULTI_STATE_DIR)` <br>
/// For "non_CBSA_residents" data directory, call: `iter_cbsa_all_files(NON_CBSA_RESIDENTS_DIR)`
pub fn iter_cbsa_all_files(subdirectory: &'static str) -> Result<std::vec::IntoIter<PathBuf>, ASPRError> {
    let mut path = get_aspr_data_path();
    path.push(CBSA_ALL_DIR);
    path.push(subdirectory);
    
    let mut files = vec![];
    let entries   = path.read_dir().map_err(|e| ASPRError::Io(e) )?;

    for entry in entries {
        let entry = entry.map_err(|e| ASPRError::Io(e) )?;
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    Ok(files.into_iter())
}

/// Returns an iterator over all the files in the ASPR "cbsa_all_work_school_household" data directory.
pub fn iter_cbsa_only_residents_files(subdirectory: &'static str) -> Result<std::vec::IntoIter<PathBuf>, ASPRError> {
    let mut path = get_aspr_data_path();
    path.push(CBSA_ONLY_RESIDENTS_DIR);
    path.push(subdirectory);
    
    let mut files = vec![];
    let entries   = path.read_dir().map_err(|e| ASPRError::Io(e) )?;

    for entry in entries {
        let entry = entry.map_err(|e| ASPRError::Io(e) )?;
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    Ok(files.into_iter())
}

/// Iterator over ASPR records in a particular ASPR data file.
pub struct ASPRRecordIterator {
    line_iter: std::io::Lines<std::io::BufReader<std::fs::File>>,
}

impl ASPRRecordIterator {
    /// Returns an iterator over the records in `${ASPR_DATA_PATH}/${ALL_STATES_DIR}/${state}.csv`
    pub fn state_population(state: USState) -> Result<Self, ASPRError> {
        let file_name = format!("{}.csv", state.to_string().to_lowercase());
        let mut path  = get_aspr_data_path();

        path.push(ALL_STATES_DIR);
        path.push(file_name);

        Self::from_path(path)
    }

    /// Returns an iterator over the records in `path`. This function is intended to be used with the
    /// `iter_*_files` functions.
    pub fn from_path(path: PathBuf) -> Result<Self, ASPRError> {
        let file          = std::fs::File::open(path.clone()).map_err(|e| ASPRError::Io(e) )?;
        let mut line_iter = std::io::BufReader::new(file).lines();

        // Skip the header row
        if line_iter.next().is_none(){
            // If there is no header row, something is wrong, so return an error.
            return Err(ASPRError::EmptyFile(path));
        }

        Ok(Self { line_iter })
    }
    
    /// Returns an iterator over all the rows of all the files in the iterator. This function is intended to be used with the
    /// `iter_*_files` functions.
    pub fn from_file_iterator(files: impl Iterator<Item=PathBuf>) 
        -> impl Iterator<Item = ASPRPersonRecord>
    {
        // Try to open each file, drop it if Err(_)
        files.filter_map(|path| ASPRRecordIterator::from_path(path).ok())
             // Each successful iterator yields records; flatten them all.
             .flat_map(|records| records)
    }
}

impl Iterator for ASPRRecordIterator {
    type Item = ASPRPersonRecord;

    /// Returns the next record in the ASPR data file. This function returns `None` on malformed data. We assume
    /// that the prepared data is well-formed.
    fn next(&mut self) -> Option<Self::Item> {
        let line          = (self.line_iter.next()?).ok()?;
        let mut part_iter = line.split(',');

        let age           = part_iter.next()?.parse::<u8>().unwrap();

        let home_id_str   = part_iter.next()?.trim();
        let home_id       = parse_fips_home_id(home_id_str).ok().map(|(_, id)| id);

        let school_id_str = part_iter.next()?.trim();
        let school_id     = parse_fips_school_id(school_id_str).ok().map(|(_, id)| id);

        let work_id_str   = part_iter.next()?.trim();
        let work_id       = parse_fips_workplace_id(work_id_str).ok().map(|(_, id)| id);

        Some(
            ASPRPersonRecord{
                age,
                home_id,
                school_id,
                work_id,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_iterator_state_population() {
        let records   = ASPRRecordIterator::state_population(USState::WY).unwrap();
        // We count the lines in the file excluding the header:
        //     583,201 - 1 = 583,200
        assert_eq!(records.count(), 583200);
    }

    #[test]
    fn test_record_iterator_from_path() {
        let data_root = get_aspr_data_path();
        let path      = data_root.join(CBSA_ALL_DIR).join("AK/Ketchikan AK.csv");
        let records   = ASPRRecordIterator::from_path(path).unwrap();
        // We count the lines in the file excluding the header:
        //     14,133 - 1 = 14,132
        assert_eq!(records.count(), 14132);
    }
    
    #[test]
    fn test_record_iterator_from_files() {
        let data_root = get_aspr_data_path();
        let paths = vec![
            data_root.join(CBSA_ALL_DIR).join("AK/Ketchikan AK.csv"),
            data_root.join(CBSA_ALL_DIR).join("TX/Vernon TX.csv"),
            data_root.join(CBSA_ONLY_RESIDENTS_DIR).join("AK/Ketchikan AK.csv"),
            data_root.join(CBSA_ONLY_RESIDENTS_DIR).join("TX/Vernon TX.csv"),
        ].into_iter();
        
        let records = ASPRRecordIterator::from_file_iterator(paths);
        
        // We sum the count of lines in each file excluding the header:
        //     14,133 + 16,606 + 13,746 + 12,973 - 4 = 57,454
        assert_eq!(records.count(), 57454);
    }

    #[test]
    fn test_iter_all_states_files() {
        let files = iter_all_states_files();
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }
    }

    #[test]
    fn test_iter_cbsa_all_files() {
        // Using a state
        let files = iter_cbsa_all_files(USState::AL.as_str());
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }

        // Using "non_CBSA_residents"
        let files = iter_cbsa_all_files(NON_CBSA_RESIDENTS_DIR);
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }

        // Using "Multi-states"
        let files = iter_cbsa_all_files(MULTI_STATE_DIR);
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }
    }

    #[test]
    fn test_iter_cbsa_only_residents_files() {
        // Using a state
        let files = iter_cbsa_only_residents_files(USState::AL.as_str());
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }

        // Using "non_CBSA_residents"
        let files = iter_cbsa_only_residents_files(NON_CBSA_RESIDENTS_DIR);
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }

        // Using "Multi-states"
        let files = iter_cbsa_only_residents_files(MULTI_STATE_DIR);
        assert!(files.is_ok());

        let files = files.unwrap().into_iter();
        for file in files {
            println!("{}", file.display());
        }
    }

    #[test]
    fn test_state_row_iter() {
        let state         = USState::AL;
        let state_records = ASPRRecordIterator::state_population(state).unwrap();

        for (idx, record) in state_records.enumerate() {
            if idx == 10 { break; }
            println!("{}", record);
        }
    }
}
