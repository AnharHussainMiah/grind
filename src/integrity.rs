use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use md5;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityData {
    files: HashMap<String, String>,
}

pub fn generate_integrity_data(dir: &Path) -> io::Result<String> {
    let mut files_hash: HashMap<String, String> = HashMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|f| f.file_name() != "integrity.json")
    {
        let path = entry.path();

        if path.is_file() {
            let mut file = fs::File::open(path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            let hash = format!("{:x}", md5::compute(&contents));
            // Store path relative to base dir
            let relative_path = path
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            files_hash.insert(relative_path, hash);
        }
    }

    let data = IntegrityData { files: files_hash };
    let json = serde_json::to_string_pretty(&data)?;
    Ok(json)
}

pub fn verify_integrity_data(dir: &Path, json_data: &str) -> io::Result<bool> {
    let parsed: IntegrityData = match serde_json::from_str(json_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Failed to parse integrity.json: {}", e);
            return Ok(false);
        }
    };

    let mut all_match = true;
    let mut total = 0;
    let mut missing = 0;
    let mut mismatched = 0;

    println!("🔍 Verifying files in directory: {}", dir.display());

    for (relative_path, expected_hash) in &parsed.files {
        total += 1;
        let full_path = dir.join(relative_path);

        if !full_path.exists() {
            println!("[MISSING] {:<60} ⛔ File does not exist", relative_path);
            missing += 1;
            all_match = false;
            continue;
        }

        let mut file = fs::File::open(&full_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let actual_hash = format!("{:x}", md5::compute(&contents));

        if &actual_hash != expected_hash {
            println!(
                "[MISMATCH] {:<60}\n           Expected: {}\n           Actual:   {}",
                relative_path, expected_hash, actual_hash
            );
            mismatched += 1;
            all_match = false;
        } else {
            println!("[OK] {} | {}", actual_hash, relative_path);
        }
    }

    println!("\n📄 Summary:");
    println!("  Total files checked : {}", total);
    println!("  Missing files       : {}", missing);
    println!("  Hash mismatches     : {}", mismatched);

    if all_match {
        println!("✅ All files passed integrity check.");
    } else {
        println!("❌ Some files failed integrity check.");
    }

    Ok(all_match)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_integrity_on_src_directory() {
        let src_path = Path::new("HelloWorld");
        assert!(src_path.exists(), "src/ directory must exist");

        let json_data =
            generate_integrity_data(src_path).expect("Failed to generate integrity data");

        let result =
            verify_integrity_data(src_path, &json_data).expect("Failed to verify integrity data");

        assert!(result, "Integrity check failed unexpectedly");
    }
}
