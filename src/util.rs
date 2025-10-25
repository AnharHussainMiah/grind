use crate::Grind;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;
use zip::ZipArchive;

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

pub fn shell_result(cmd: &str) -> Result<String, String> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        return Ok(stdout.to_string());
    }

    if !stderr.trim().is_empty() {
        return Err(stderr.to_string());
    }

    Err("Error: Unable both stdout and stderror failed..".to_string())
}

pub fn ls_with_ext(dir: &str, extension: &str) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension()
                && ext == extension
            {
                if let Some(file_str) = path.to_str() {
                    files.push(file_str.to_string());
                }
            }
        }
    }

    Ok(files)
}

pub fn unzip_file(zip_path: &Path, destination: &Path) -> zip::result::ZipResult<()> {
    // Open the zip file
    let zip_file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination.join(file.name());

        // If it's a directory, create it
        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            // If it's a file, create any missing parent directories, then write it
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Optional: Set permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

use std::cmp::Ordering;

/// Known Maven qualifier ranking
fn qualifier_rank(q: &str) -> i32 {
    match q.to_ascii_lowercase().as_str() {
        "snapshot" => 1,
        "alpha" | "a" => 2,
        "beta" | "b" => 3,
        "milestone" | "m" => 4,
        "rc" | "cr" => 5,
        "" | "final" | "ga" | "release" => 6,
        "sp" => 7,
        _ => 8, // unknown qualifiers
    }
}

/// Split a token into numeric part, qualifier letters, qualifier number
fn split_token(token: &str) -> (u64, String, u64) {
    let mut digits = String::new();
    let mut letters = String::new();
    let mut qualifier_number = String::new();
    let mut in_letters = false;

    for c in token.chars() {
        if c.is_ascii_digit() && !in_letters {
            digits.push(c);
        } else if c.is_ascii_alphabetic() {
            in_letters = true;
            letters.push(c);
        } else if c.is_ascii_digit() && in_letters {
            qualifier_number.push(c);
        }
    }

    let number = digits.parse::<u64>().unwrap_or(0);
    let qnum = qualifier_number.parse::<u64>().unwrap_or(0);

    (number, letters, qnum)
}

/// Extract tokens from version string
fn extract_tokens(v: &str) -> Vec<(u64, String, u64)> {
    v.split(|c| c == '.' || c == '-' || c == '_')
        .map(split_token)
        .collect()
}

/// Compare two Maven-style versions with proper qualifiers
pub fn compare_maven_versions(v1: &str, v2: &str) -> Ordering {
    let tokens1 = extract_tokens(v1);
    let tokens2 = extract_tokens(v2);

    let max_len = tokens1.len().max(tokens2.len());

    for i in 0..max_len {
        let (n1, q1, qn1) = tokens1.get(i).cloned().unwrap_or((0, "".into(), 0));
        let (n2, q2, qn2) = tokens2.get(i).cloned().unwrap_or((0, "".into(), 0));

        // Compare main numeric
        if n1 != n2 {
            return n1.cmp(&n2);
        }

        // Compare qualifier rank
        let r1 = qualifier_rank(&q1);
        let r2 = qualifier_rank(&q2);
        if r1 != r2 {
            return r1.cmp(&r2);
        }

        // Compare qualifier numeric
        if qn1 != qn2 {
            return qn1.cmp(&qn2);
        }

        // Fallback: lex comparison for unknown qualifiers
        if q1 != q2 {
            return q1.cmp(&q2);
        }
    }

    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_versions() {
        let cases = vec![
            ("9.4.6.v20170531", "9.4.6.v20170530", Ordering::Greater),
            ("9.4.6.v20170531", "9.4.6", Ordering::Greater),
            ("9.4.6.v20170531", "9.4.6.v20170531", Ordering::Equal),
            ("1.0-RC1", "1.0-RC2", Ordering::Less),
            ("1.0-RC1", "1.0", Ordering::Less),
            ("1.0.0", "1.0", Ordering::Equal),
            ("1.0-SNAPSHOT", "1.0", Ordering::Less),
            ("2.0", "1.9.9", Ordering::Greater),
        ];

        for (a, b, expected) in cases {
            let result = compare_maven_versions(a, b);
            assert_eq!(
                result, expected,
                "compare_maven_versions('{}', '{}') returned {:?}, expected {:?}",
                a, b, result, expected
            );
        }
    }
}
