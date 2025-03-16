use std::{fs::File, io::BufReader, path::Path};

use anyhow::Error;
use apiversions::SupportedVersionsKey;

pub mod apiversions;

pub mod describetopic;

/// Checks if a given version is supported for a specific key.
///
/// This function reads a JSON file (`supported_versions.json`) which contains a list
/// of supported version ranges for various keys. It checks if the provided `key` and
/// `version` fall within a valid supported version range.
///
/// # Arguments
///
/// * `key` - An `i16` representing the key for which the version is being checked.
/// * `version` - An `i16` representing the version to be validated.
///
/// # Returns
///
/// * `Ok(true)` if the version is supported for the given key.
/// * `Ok(false)` if the version is not supported for the given key.
/// * `Err(Error)` in case of any errors reading the JSON file or parsing the data.
///
/// # Errors
///
/// This function may return an error if there is an issue opening the file, reading it,
/// or parsing the JSON content. These errors are propagated using the `anyhow::Error` type.
pub fn is_version_supported<P: AsRef<Path>>(
    path: P,
    key: i16,
    version: i16,
) -> Result<bool, Error> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);

    let data: Vec<SupportedVersionsKey> = serde_json::from_reader(reader)?;

    Ok(data
        .iter()
        .filter(|val| val.key == key && (version >= val.min && version <= val.max))
        .last()
        .is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs::{remove_file, File};
    use std::io::Write;

    #[derive(Serialize, Deserialize, Debug)]
    struct SupportedVersionsKey {
        key: i16,
        min: i16,
        max: i16,
    }

    fn create_mock_supported_versions_json() {
        let mock_data = vec![
            SupportedVersionsKey {
                key: 1,
                min: 1,
                max: 5,
            },
            SupportedVersionsKey {
                key: 2,
                min: 3,
                max: 7,
            },
        ];

        let file =
            File::create("supported_versions_valid.json").expect("Failed to create mock file");
        serde_json::to_writer(file, &mock_data).expect("Failed to write mock data");
    }

    #[test]
    fn test_version_supported() {
        create_mock_supported_versions_json();

        let result = is_version_supported("supported_versions_valid.json", 1, 3);
        assert!(result.unwrap());

        let result = is_version_supported("supported_versions_valid.json", 1, 6);
        assert!(!result.unwrap());
        let result = is_version_supported("supported_versions_valid.json", 2, 7);
        assert!(result.unwrap());

        let result = is_version_supported("supported_versions_valid.json", 2, 2);
        assert!(!result.unwrap());
    }

    #[test]
    fn test_version_not_supported() {
        create_mock_supported_versions_json();

        let result = is_version_supported("supported_versions_valid.json", 1, 6);
        assert!(!result.unwrap());

        let result = is_version_supported("supported_versions_valid.json", 2, 8);
        assert!(!result.unwrap());

        let _ = remove_file("supported_versions_valid.json");
    }

    #[test]
    fn test_invalid_file() {
        let result = is_version_supported("unexisting_file.csv", 1, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_parse_error() {
        let mut file =
            File::create("supported_versions_marlformed.json").expect("Failed to create mock file");
        file.write_all(b"invalid json")
            .expect("Failed to write invalid data");

        let result = is_version_supported("supported_versions_marlformed.json", 1, 3);
        assert!(result.is_err());
        let _ = remove_file("supported_versions_marlformed.json");
    }

    #[test]
    fn test_empty_file() {
        let file =
            File::create("supported_versions_empty.json").expect("Failed to create mock file");
        drop(file);

        let result = is_version_supported("supported_versions_empty.json", 1, 3);
        assert!(result.is_err());
        let _ = remove_file("supported_versions_empty.json");
    }
}
