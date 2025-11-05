use crate::Grind;
use flate2::read::GzDecoder;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tar::Archive;
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

#[allow(dead_code)]
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

pub fn shell_stream(cmd: &str) -> std::io::Result<()> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .arg("--color=auto")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Handle stderr in a background thread
    std::thread::spawn(move || {
        for line in stderr_reader.lines().flatten() {
            eprintln!("[stderr] {}", line);
        }
    });

    // Handle stdout on the main thread
    for line in stdout_reader.lines().flatten() {
        println!("{}", line);
    }

    let status = child.wait()?;
    println!("Process exited with: {}", status);

    Ok(())
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
    let zip_file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
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

pub fn expand_tilde(path: &str) -> Option<PathBuf> {
    if path.starts_with('~') {
        let home = env::var("HOME").ok()?;
        return Some(PathBuf::from(home).join(path[1..].trim_start_matches('/')));
    }
    Some(PathBuf::from(path))
}

pub fn dir_exists(path_str: &str) -> bool {
    if let Some(expanded_path) = self::expand_tilde(path_str) {
        return match std::fs::metadata(&expanded_path) {
            Ok(meta) => meta.is_dir(),
            Err(e) => {
                eprintln!("Error accessing '{}': {}", expanded_path.display(), e);
                return false;
            }
        };
    }
    false
}

pub fn create_symlink(target: &str, link_name: &str) -> bool {
    let target_path = match expand_tilde(target) {
        Some(p) => p,
        None => {
            eprintln!("Failed to expand target path '{}'", target);
            return false;
        }
    };

    let link_path = match expand_tilde(link_name) {
        Some(p) => p,
        None => {
            eprintln!("Failed to expand link path '{}'", link_name);
            return false;
        }
    };

    let _ = std::fs::remove_file(&link_path);

    if let Err(e) = symlink(&target_path, &link_path) {
        eprintln!("Failed to create symlink '{}': {}", link_path.display(), e);
        return false;
    }
    true
}

pub fn extract_tar_gz(
    archive_path: &Path,
    target_dir: &Path,
    rename_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

    // Record existing entries before extraction, a bit silly but don't know how else to
    // figure out the extracted folder name
    let before: HashSet<String> = fs::read_dir(target_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let tar_gz = File::open(archive_path).map_err(|e| e.to_string())?;
    let decompressor = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);

    archive.unpack(target_dir).map_err(|e| e.to_string())?;

    let after: HashSet<String> = fs::read_dir(target_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let new_entries: Vec<_> = after.difference(&before).cloned().collect();

    if !new_entries.is_empty() {
        let path = target_dir.join(new_entries[0].clone());
        if path.is_dir() {
            fs::rename(path, rename_dir).map_err(|e| e.to_string())?;
        }
    } else {
        return Err("Unable to find extracted tar file!".to_string());
    }

    Ok(())
}

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

fn extract_tokens(v: &str) -> Vec<(u64, String, u64)> {
    v.split(|c| c == '.' || c == '-' || c == '_')
        .map(split_token)
        .collect()
}

pub fn compare_maven_versions(v1: &str, v2: &str) -> Ordering {
    let tokens1 = extract_tokens(v1);
    let tokens2 = extract_tokens(v2);

    let max_len = tokens1.len().max(tokens2.len());

    for i in 0..max_len {
        let (n1, q1, qn1) = tokens1.get(i).cloned().unwrap_or((0, "".into(), 0));
        let (n2, q2, qn2) = tokens2.get(i).cloned().unwrap_or((0, "".into(), 0));

        if n1 != n2 {
            return n1.cmp(&n2);
        }

        let r1 = qualifier_rank(&q1);
        let r2 = qualifier_rank(&q2);
        if r1 != r2 {
            return r1.cmp(&r2);
        }

        if qn1 != qn2 {
            return qn1.cmp(&qn2);
        }

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
            ("3.5.3", "4.0.0-M3", Ordering::Less),
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
