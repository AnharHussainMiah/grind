use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Instant, SystemTime};

use zip::{ZipArchive, ZipWriter, write::FileOptions};

pub struct FatJarConfig<'a> {
    pub output_jar: &'a Path,
    pub classes_dir: &'a Path,
    pub libs_dir: &'a Path,
    pub main_class: &'a str,
    pub group_id: &'a str,
    pub artifact_id: &'a str,
}

/// Build a runnable "fat jar" (includes all dependencies).
pub fn build_fat_jar(config: &FatJarConfig) -> io::Result<()> {
    let start_time = Instant::now();

    let mut seen_entries = HashSet::new();
    let mut merged_resources: HashMap<String, Vec<u8>> = HashMap::new();

    let file = File::create(config.output_jar)?;
    let mut writer = ZipWriter::new(file);

    let options: FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    println!("🧩 Starting fat jar build...");
    println!(" → Output: {}", config.output_jar.display());
    println!(" → Classes: {}", config.classes_dir.display());
    println!(" → Libs: {}", config.libs_dir.display());
    println!(" → Main-Class: {}", config.main_class);

    // 1. Manifest
    let manifest = generate_manifest(config);
    writer.start_file("META-INF/MANIFEST.MF", options)?;
    writer.write_all(manifest.as_bytes())?;
    seen_entries.insert("META-INF/MANIFEST.MF".into());
    println!("+ manifest: META-INF/MANIFEST.MF");

    // 2. Application classes/resources
    let mut file_count = 0;
    add_directory_classes(
        config.classes_dir,
        &mut writer,
        &mut seen_entries,
        &mut file_count,
    )?;

    // 3. Dependencies with progress bar showing count
    let jars: Vec<_> = fs::read_dir(config.libs_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "jar").unwrap_or(false))
        .collect();

    let total = jars.len();
    if total > 0 {
        println!("📦 Merging {} dependency jar(s)...", total);
    }

    for (i, jar_path) in jars.iter().enumerate() {
        let percent = ((i + 1) * 100) / total.max(1);
        print!(
            "\r   [{}/{} | {:3}%] {}",
            i + 1,
            total,
            percent,
            jar_path.file_name().unwrap().to_string_lossy()
        );
        io::stdout().flush()?;
        merge_jar(
            jar_path,
            &mut writer,
            &mut seen_entries,
            &mut merged_resources,
        )?;
    }

    if total > 0 {
        println!(); // newline after progress bar
    }

    // 4. Write merged service/spring resources
    for (name, data) in merged_resources {
        if seen_entries.insert(name.clone()) {
            writer.start_file(&name, options)?;
            writer.write_all(&data)?;
            println!("+ merged: {}", name);
        }
    }

    writer.finish()?;

    let elapsed = start_time.elapsed();
    println!("✅ Fat jar created: {}", config.output_jar.display());
    println!("📊 Summary:");
    println!("   → {} class/resource files", file_count);
    println!("   → {} dependency jars merged", total);
    println!(
        "⏱ Total build time: {}.{:03} seconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    Ok(())
}

fn generate_manifest(config: &FatJarConfig) -> String {
    let now = SystemTime::now();
    let date = chrono::DateTime::<chrono::Utc>::from(now).to_rfc2822();

    format!(
        "Manifest-Version: 1.0\n\
         Main-Class: {main}\n\
         Implementation-Title: {artifact}\n\
         Implementation-Vendor-Id: {group}\n\
         Built-By: {user}\n\
         Build-Jdk: {jdk}\n\
         Implementation-Version: 1.0.0\n\n",
        main = config.main_class,
        artifact = config.artifact_id,
        group = config.group_id,
        user = "grind",
        jdk = std::env::var("JAVA_HOME").unwrap_or_else(|_| "unknown".into()),
    )
}

fn add_directory_classes(
    dir: &Path,
    writer: &mut ZipWriter<File>,
    seen: &mut HashSet<String>,
    file_count: &mut usize,
) -> io::Result<()> {
    let options: FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let rel_path = entry
                .path()
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");

            if seen.insert(rel_path.clone()) {
                writer.start_file(rel_path.clone(), options)?;
                let mut f = File::open(entry.path())?;
                io::copy(&mut f, writer)?;
                *file_count += 1;

                if rel_path.ends_with(".class") {
                    println!("+ class: {}", rel_path);
                } else {
                    println!("+ resource: {}", rel_path);
                }
            }
        }
    }
    Ok(())
}

fn merge_jar(
    jar_path: &Path,
    writer: &mut ZipWriter<File>,
    seen: &mut HashSet<String>,
    merged: &mut HashMap<String, Vec<u8>>,
) -> io::Result<()> {
    let file = File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;

    let options: FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if entry.is_dir() || name == "META-INF/MANIFEST.MF" || is_signature_file(&name) {
            continue;
        }

        if is_mergeable(&name) {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            merge_resource(merged, &name, &buf);
        } else if seen.insert(name.clone()) {
            writer.start_file(name.clone(), options)?;
            io::copy(&mut entry, writer)?;
        }
    }

    Ok(())
}

fn is_signature_file(name: &str) -> bool {
    let upper = name.to_uppercase();
    upper.starts_with("META-INF/")
        && [".SF", ".RSA", ".DSA"]
            .iter()
            .any(|ext| upper.ends_with(ext))
}

fn is_mergeable(name: &str) -> bool {
    name.starts_with("META-INF/services/")
        || name == "META-INF/spring.factories"
        || name.starts_with("META-INF/spring/")
}

fn merge_resource(merged: &mut HashMap<String, Vec<u8>>, name: &str, buf: &[u8]) {
    let entry = merged.entry(name.to_string()).or_default();
    entry.extend_from_slice(buf);
    if !entry.ends_with(b"\n") {
        entry.push(b'\n');
    }
}
