use std::path::Path;

use crate::util;

pub fn run_tests(tests: Vec<String>) {
    if !self::check_plugin_exists() {
        self::download_test_plugin();
    }

    if self::check_plugin_integrity() {
        // TODO Run tests
    } else {
        println!("❌ the TestTube plugin is corrupted, try deleting the `TestTube/` folder");
    }
}

fn check_plugin_exists() -> bool {
    // TODO: plugins/TestTube/integrity.json
    false
}

fn download_test_plugin() {
    println!("🌎 Downloading TestTube plugin...");


    self::unzip_test_plugin("TestTube.zip".to_string());
}

fn unzip_test_plugin(filename: String) {
    println!("🗜️ Extracting TestTube plugin...");

    let zip_path = Path::new(&filename);
    let destination = Path::new("plugins");

    match util::unzip_file(zip_path, destination) {
        Ok(()) => println!("✅ Extraction complete!"),
        Err(e) => eprintln!("❌ Error during extraction: {}", e),
    }
}

fn check_plugin_integrity() -> bool {
    // TODO
    false
}
