use crate::Grind;
use std::fs;
use std::process::Command;

pub fn parse_grind_file() -> Option<Grind> {
    let grind_raw = fs::read_to_string("grind.yml").unwrap();
    let parsed: Grind = serde_yaml::from_str(&grind_raw).unwrap();
    Some(parsed)
}

pub fn shell(cmd: &str) -> String {
    // print!("[DEBUG-CMD] {}", cmd);
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();

    if !stdout.trim().is_empty() {
        result.push_str(&stdout);
    }

    if !stderr.trim().is_empty() {
        result.push_str("\n⚠️ Error:\n");
        result.push_str(&stderr);
    }

    result.trim().to_string()
}

pub fn ls_with_ext(dir: &str, extension: &str) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    if let Some(file_str) = path.to_str() {
                        files.push(file_str.to_string());
                    }
                }
            }
        }
    }

    Ok(files)
}
