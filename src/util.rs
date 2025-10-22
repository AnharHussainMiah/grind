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
