#[cfg(test)]
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// path: path to dir or filename
/// returns filename, content
pub fn get_test_files(path: &PathBuf) -> Result<HashMap<String, String>> {
    if path.is_dir() {
        let mut res: HashMap<String, String> = HashMap::new();
        for dir in fs::read_dir(path)? {
            let entry = dir?;
            let entry = entry.path();
            if entry.is_dir() {
                unreachable!("Do not support a recursive path!")
            }
            res.insert(
                entry.file_name().unwrap().to_string_lossy().into(),
                fs::read_to_string(&entry)?,
            );
        }
        return Ok(res);
    }
    let mut map = HashMap::new();
    map.insert(
        path.file_name().unwrap().to_string_lossy().into(),
        fs::read_to_string(path)?,
    );
    Ok(map)
}

pub fn compare_files(expected: &HashMap<String, String>, found: &HashMap<String, String>) {
    for (filename, expected_content) in expected {
        let found_content = found.get(filename);
        assert!(found_content.is_some());
        assert_eq!(
            found_content.unwrap().as_str(),
            expected_content.as_str(),
            "The found file does not match the expected file!"
        );
    }
}
