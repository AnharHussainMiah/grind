use crate::util;
use crate::util::GrindPath;
use crate::util::shell_custom_path;
use futures_util::StreamExt;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use serde_json::from_str;
use std::cmp::min;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct JavaMetaData {
    available_releases: Vec<i32>,
}

pub async fn list() {
    /* ---------------------------------------------------------------------------------------------
    call remote API and show all available JDK versions
    --------------------------------------------------------------------------------------------- */
    if let Err(s) = self::get_list().await {
        println!("❌ Unable to fetch the list of JDK verions: {}", s);
    }
}

pub fn current() {
    /* ---------------------------------------------------------------------------------------------
    if the symlink exits, check where it points to and show that as the current
    otherwise check the system wide version (if it exists) and list that version

    make sure to let the user know if the current version is the system or grind managed version
    --------------------------------------------------------------------------------------------- */
    match self::run_jdk_checks() {
        Ok(is_grind_jdk) => {
            let mut include_grind_path = false;

            let managed_jdk = if is_grind_jdk {
                include_grind_path = true;
                "✅ [Grind Managed JDK]"
            } else {
                "🖥️  [System Installed JDK]"
            };

            if let Ok(version) = self::get_java_version(include_grind_path) {
                println!("{} | v{}", managed_jdk, version);
            } else {
                println!("❌ Unable to detect any Java on this system");
            }
        }
        Err(e) => {
            println!("Error, unable to inspect grind JDK! {}", e);
        }
    }
}

async fn get_list() -> Result<(), String> {
    let url = "https://api.adoptium.net/v3/info/available_releases";

    println!("🌎 Fetching metadata");

    let json = reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let metadata: JavaMetaData = from_str(&json).map_err(|e| e.to_string())?;

    if !metadata.available_releases.is_empty() {
        println!(
            "\nFound {} JDK versions:\n",
            metadata.available_releases.len()
        );
    }
    for version in metadata.available_releases {
        println!(" - v{}", version);
    }

    Ok(())
}

pub async fn _use(version: String) {
    match self::get_jdk_detail(&version).await {
        Ok(download_link) => {
            /* -------------------------------------------------------------------------------------
            we know this is a valid version, we need to run some idempotent operations:
            create the directory (if one doesn't already exist)
            download the sdk (unless it already exists)
            create the symlink (overwrite even if one exists) e.g ~/.grind/jdks/current -> ./v22/bin
            make sure that the ~/.bashrc contains the PATH if not add it
            ------------------------------------------------------------------------------------- */
            if let Err(e) = self::run_install(&version, &download_link).await {
                println!("❌ Unable to setup and install JDK: {}", e);
            } else {
                println!("✅ JDK v{} setup is completed!", &version);
            }
        }
        Err(error) => println!("❌ Unable to fetch JDK version metadata: {}", error),
    }
}

async fn run_install(version: &String, download_link: &String) -> Result<(), String> {
    self::create_jdk_dir().map_err(|e| e.to_string())?;
    self::download_sdk(&version, download_link.to_string())
        .await
        .map_err(|e| e.to_string())?;
    self::create_symlink(version).map_err(|e| e.to_string())?;
    self::set_bash_rc_path().map_err(|e| e.to_string())?;
    Ok(())
}

fn run_jdk_checks() -> Result<bool, String> {
    let is_jdk = self::is_jdk_dir_exist()?;
    let is_symlink = self::is_symlink_exist()?;
    let is_bashrc = self::is_bashrc_exist()?;

    return Ok(is_jdk && is_bashrc && is_symlink && is_bashrc);
}

fn is_jdk_dir_exist() -> Result<bool, String> {
    Ok(util::dir_exists(&"~/.grind/jdks"))
}

fn is_symlink_exist() -> Result<bool, String> {
    let full_path =
        util::expand_tilde(&"~/.grind/jdks/current").ok_or("unable to expand tilde path!")?;
    Ok(std::fs::metadata(&full_path).is_ok())
}

fn is_bashrc_exist() -> Result<bool, String> {
    let bashrc_path = util::expand_tilde("~/.bashrc").ok_or("unable to expand tilde path")?;
    let bashrc = fs::read_to_string(&bashrc_path).map_err(|e| e.to_string())?;

    Ok(bashrc.contains("# GRIND-JDK-PATH"))
}

fn get_java_version(include_grind_path: bool) -> Result<String, String> {
    let grind_path_option = if include_grind_path {
        GrindPath::Include
    } else {
        GrindPath::Exlude
    };

    let out = shell_custom_path("java --version", grind_path_option);

    let re = Regex::new(r#"(\d+\.\d+\.\d+.*\w)"#).map_err(|e| e.to_string())?;

    if let Some(caps) = re.captures(&out) {
        Ok(format!("{}", &caps[1]))
    } else {
        Err("Could not determine Java version.".to_string())
    }
}

fn create_jdk_dir() -> Result<(), String> {
    let full_path =
        util::expand_tilde(&format!("~/.grind/jdks")).ok_or("unable to expand tilde path!")?;
    let _ = std::fs::create_dir_all(full_path).map_err(|e| e.to_string())?;
    println!("✅ valid JDK directory...");
    Ok(())
}

async fn download_sdk(version: &String, url: String) -> Result<(), String> {
    let sdk = format!("~/.grind/jdks/v{}", version);
    if !util::dir_exists(&sdk) {
        println!("==> JDK v{} not yet installed, downloading...", version);

        let grind_sdk_path =
            util::expand_tilde("~/.grind/jdks/").ok_or("Unable to expand grind JDK tilde")?;

        let filename = url.rsplit('/').next().ok_or("Unable to extract filename")?;
        let local_path = format!("{}{}", grind_sdk_path.display(), filename);

        println!("📥 Downloading: {}", &url);
        self::download_with_progress(&url, &local_path).await?;
        println!("🗜️  Extracting archive, please wait!...");
        util::extract_tar_gz(
            &Path::new(&local_path),
            &grind_sdk_path,
            &Path::new(&format!("{}v{}", grind_sdk_path.display(), version)),
        )?;
        return Ok(());
    }
    println!("✅ valid JDK package...");
    Ok(())
}

async fn download_with_progress(url: &str, output_path: &str) -> Result<(), String> {
    let client = Client::new();

    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let mut file =
        File::create(output_path).or(Err(format!("Failed to create file '{}'", output_path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let percentage = (downloaded as f64 / total_size as f64) * 100.0;
        print!(
            "\rDownloaded: {:.2}% ({}/{})",
            percentage,
            self::format_bytes(downloaded),
            self::format_bytes(total_size)
        );
        io::stdout().flush().map_err(|e| e.to_string())?;
    }

    println!("");
    println!("{}", &format!("✅ Finished! downloaded to {}", output_path));
    return Ok(());
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let bytes_f = bytes as f64;

    if bytes_f >= TB {
        format!("{:.2} TB", bytes_f / TB)
    } else if bytes_f >= GB {
        format!("{:.2} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.2} KB", bytes_f / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn create_symlink(version: &String) -> Result<(), String> {
    if !util::create_symlink(
        &format!("~/.grind/jdks/v{}/bin", version),
        &format!("~/.grind/jdks/current"),
    ) {
        Err("Error, unable to create symlink!".to_string())
    } else {
        println!("✅ valid symlink...");
        Ok(())
    }
}

fn set_bash_rc_path() -> Result<(), String> {
    let bashrc_path = util::expand_tilde("~/.bashrc").ok_or("unable to expand tilde path")?;
    let bashrc = fs::read_to_string(&bashrc_path).map_err(|e| e.to_string())?;

    if bashrc.contains("# GRIND-JDK-PATH") {
        println!("✅ valid PATH in bashrc...");
        return Ok(());
    } else {
        // todo add grind path export + messaging
        let content = r#"
# GRIND-JDK-PATH
export PATH="$HOME/.grind/jdks/current:$PATH"
# GRIND-JDK-PATH
        "#;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&bashrc_path)
            .map_err(|e| e.to_string())?;

        file.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;

        println!("ℹ️  You will need to reload your shell for new PATH to take affect");
        println!("✔ Updated .bashrc — ⚡ WARNING: restart your terminal");
        return Ok(());
    }
}

async fn get_jdk_detail(version: &String) -> Result<String, String> {
    let version = version.trim_start_matches('v');
    let os = self::map_os(std::env::consts::OS);
    let arch = self::map_arch(std::env::consts::ARCH);

    println!("Using autodetected: OS ({}) | Architecture ({}).", os, arch);

    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os={}&vendor=eclipse",
        version, arch, os
    );

    println!("🌎 Fetching metadata for version {}", version);

    let json = reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let v: Vec<Value> = from_str(&json).map_err(|e| e.to_string())?;
    if !v.is_empty() {
        let link = &v[0]["binary"]["package"]["link"];
        return Ok(link.to_string().replace("\"", ""));
    } else {
        return Err(format!(
            "❌ Could not extract metadata for v{} , are you sure this is a valid vesion?",
            version
        ));
    }
}

fn map_os(os: &str) -> String {
    os.replace("macos", "mac")
}

fn map_arch(arch: &str) -> String {
    arch.replace("x86_64", "x64").replace("x86", "x32")
}

pub fn remove() {
    /* ---------------------------------------------------------------------------------------------
    nuke the symlink
    remove from path
    keep anything downloaded, if the user wants to "setup" again, no point re-downloading again
    --------------------------------------------------------------------------------------------- */
    match self::run_destroy() {
        Ok(_) => {
            println!("💣 Grind Managed JDK Destroyed!");
            println!("✔ Updated .bashrc — ⚡ WARNING: restart your terminal");
        }
        Err(e) => {
            println!("Error, unable to remove Grind managed JDK! {}", e);
        }
    }
}

fn run_destroy() -> Result<(), String> {
    let source = util::expand_tilde("~/.bashrc").ok_or("unable to expand tilde path")?;
    let backup = util::expand_tilde("~/.bashrc.bak").ok_or("unable to expand tilde path")?;

    std::fs::copy(&source, backup).map_err(|e| e.to_string())?;
    let bashrc = fs::read_to_string(&source).map_err(|e| e.to_string())?;
    let chunks: Vec<&str> = bashrc.split("# GRIND-JDK-PATH").collect();

    if !chunks.is_empty() {
        fs::write(source, chunks[0]).map_err(|e| e.to_string())?;
    }

    Ok(())
}
