use std::path::Path;

use crate::BuildTarget;
use crate::Grind;
use crate::build;
use crate::handle_validate_integrity;
use crate::util;
use crate::util::shell;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;

pub async fn run_tests(grind: Grind, tests: Vec<String>) {
    if !self::check_plugin_exists() {
        let _ = self::download_test_plugin().await;
    }

    if self::check_plugin_integrity() {
        // TODO: at the moment we're NOT passing any compiler flags
        build::execute_build(&grind, BuildTarget::IncludeTest, String::new());
        let args = tests.join(" ");
        let cmd = format!(
            "java -cp \"target:target/test:libs/*:plugins/TestTube/libs/*:plugins/TestTube/TestTube.jar\" org.grind.TestTube {}",
            args
        );
        let out = shell(&cmd);
        println!("{}", out);
    } else {
        println!("❌ the TestTube plugin is corrupted, try deleting the `TestTube/` folder");
    }
}

fn check_plugin_exists() -> bool {
    Path::new("plugins/TestTube/integrity.json").exists()
}

async fn download_test_plugin() -> Result<(), String> {
    println!("🌎 Downloading TestTube plugin...");

    let resp = reqwest::get(
        "https://github.com/AnharHussainMiah/TestTube/releases/download/v0.1.95/TestTubeFinal.zip",
    )
    .await
    .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "⚠️ Unable to download, HTTP Status Code: {}",
            resp.status()
        ));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let mut file = File::create("TestTube.zip")
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;

    let _ = file.sync_data().await;

    task::spawn_blocking(move || {
        self::unzip_test_plugin("TestTube.zip".to_string());
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn unzip_test_plugin(filename: String) {
    println!("🗜️ Extracting TestTube plugin...");

    let zip_path = Path::new(&filename);
    let destination = Path::new("plugins");

    match util::unzip_file(zip_path, destination) {
        Ok(()) => println!("✅ Extraction complete!"),
        Err(e) => eprintln!("❌ Error during extraction: {}", e),
    }

    if let Err(e) = std::fs::remove_file("TestTube.zip") {
        eprintln!("Error: {}", e);
    }
}

fn check_plugin_integrity() -> bool {
    match handle_validate_integrity("plugins/TestTube".into()) {
        Ok(_) => true,
        Err(_) => false,
    }
}
