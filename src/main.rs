use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

mod build;
mod config;
mod install;
mod integrity;
mod lock;
mod manage;
mod metadata;
mod mock;
mod pom;
mod run;
mod scaffold;
mod tasks;
mod tests;
mod uberjar;
mod util;

use crate::build::BuildTarget;
use crate::config::Grind;

const LOGO: &str = r#"
                     /$$                 /$$
                    |__/                | $$
  /$$$$$$   /$$$$$$  /$$ /$$$$$$$   /$$$$$$$
 /$$__  $$ /$$__  $$| $$| $$__  $$ /$$__  $$
| $$  \ $$| $$  \__/| $$| $$  \ $$| $$  | $$
| $$  | $$| $$      | $$| $$  | $$| $$  | $$
|  $$$$$$$| $$      | $$| $$  | $$|  $$$$$$$
 \____  $$|__/      |__/|__/  |__/ \_______/
 /$$  \ $$                                  
|  $$$$$$/                                  
 \______/                                   

        - "Java builds, without the headache"
                    v0.7.4
"#;

#[derive(Parser, Debug)]
#[command(author = "Anhar Miah", version, about = LOGO, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scaffolds a new Java project with a grind.yml file
    New {
        /// Name of the project to create <GroupId>/<ArtifactId> e.g com.example/HelloWorld
        name: String,
    },
    /// Download all the external libraries and dependencies as defined in the grind.yml
    Install,
    /// Compile the project and builds a jar file.
    Build {
        /// the defined profile to build with, these include compiler flags, and environment variables
        profile: Option<String>,
    },
    /// Compile and run the project
    Run {
        /// the defined profile to run with, these include compiler flags, and environment variables
        profile: Option<String>,
    },
    /// Adds a dependency to the project's grind.yml
    Add {
        /// List of dependencies to add (e.g., 'io.javalin/javalin org.posgresql/postgresql')
        deps: Vec<String>,
    },
    /// Removes a dependency from the project's grind.yml
    Remove {
        /// List of dependencies to remove
        deps: Vec<String>,
    },
    /// Run a custom task as defiend in the grind.yml, e.g grind task clean
    Task { job: String },
    /// Create the integrity file or validate one for plugins/packages
    Integrity {
        #[command(subcommand)]
        integrity: IntegritySubcommand,
    },
    /// Run Tests
    Test { tests: Vec<String> },
    /// Packages compiled classes and all dependency jars into a single runnable JAR, also known as a "Fat Jar" or "Uberjar"
    Bundle {
        /// the defined profile to run with, these include compiler flags, and environment variables
        profile: Option<String>
    },
}

#[derive(Subcommand, Debug)]
enum IntegritySubcommand {
    /// Generate integrity.json inside the directory
    Generate {
        /// Path to the directory to hash
        dir: PathBuf,
    },
    /// Validate the integrity of the directory using integrity.json
    Validate {
        /// Path to the directory to validate
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => self::handle_new(&name),
        Commands::Build { profile } => self::handle_build(profile),
        Commands::Install => self::handle_install().await,
        Commands::Run { profile } => self::handle_run(profile),
        Commands::Add { deps } => self::handle_add(deps).await,
        Commands::Remove { deps } => self::handle_remove(deps).await,
        Commands::Task { job } => self::handle_task(job),
        Commands::Integrity { integrity } => match integrity {
            IntegritySubcommand::Generate { dir } => {
                let _ = self::handle_generate_integrity(dir);
            }
            IntegritySubcommand::Validate { dir } => {
                let _ = self::handle_validate_integrity(dir);
            }
        },
        Commands::Test { tests } => self::handle_tests(tests).await,
        Commands::Bundle {profile } => self::handle_bundle(profile),
    }
}

fn handle_new(name: &str) {
    if let Ok((namespace, artifact_id)) = self::parse_project_name(name) {
        let folder_path = Path::new(artifact_id);

        if !folder_path.exists() || !folder_path.is_dir() {
            scaffold::create(namespace, artifact_id);
        } else {
            println!(
                "⚠️ Sorry project folder '{}' already exists, exiting...",
                artifact_id
            );
        }
    } else {
        println!(
            "⚠️ Sorry '{}' is not a valid project name, requires a namespace and artifactId, e.g com.example/HelloWorld",
            name
        );
    }
}

fn handle_build(profile: Option<String>) {
    if let Some(grind) = util::parse_grind_file() {
        let mut flags = String::new();

        if let Some(profile) = profile {
            flags = self::get_flags(&grind, profile);
        }

        build::execute_build(&grind, BuildTarget::IncludeJar, flags);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

async fn handle_install() {
    if let Some(grind) = util::parse_grind_file() {
        install::execute_install(grind).await;
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn handle_run(profile: Option<String>) {
    if let Some(grind) = util::parse_grind_file() {
        run::execute_run(grind, profile);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn handle_task(job: String) {
    if let Some(grind) = util::parse_grind_file() {
        tasks::execute_task(grind, job);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

async fn handle_add(deps: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        manage::execute_add(grind, deps).await;
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

async fn handle_remove(deps: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        manage::execute_remove(grind, deps).await;
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn parse_project_name(input: &str) -> Result<(&str, &str), &'static str> {
    let mut parts = input.split('/');

    match (parts.next(), parts.next(), parts.next()) {
        (Some(first), Some(second), None) => Ok((first, second)),
        _ => Err("⚠️ Input must contain exactly one '/' and two non-empty parts"),
    }
}

fn handle_generate_integrity(dir: PathBuf) -> Result<(), String> {
    let json = integrity::generate_integrity_data(&dir).map_err(|e| e.to_string())?;
    let integrity_file = dir.join("integrity.json");
    let mut file = File::create(&integrity_file).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    println!("✅ Integrity data written to {}", integrity_file.display());
    Ok(())
}

pub fn handle_validate_integrity(dir: PathBuf) -> Result<(), String> {
    let integrity_file = dir.join("integrity.json");
    if !integrity_file.exists() {
        eprintln!("❌ integrity.json not found in {}", dir.display());
        return Err("missng integrity.json".to_string());
    }

    let mut json_data = String::new();
    File::open(&integrity_file)
        .map_err(|e| e.to_string())?
        .read_to_string(&mut json_data)
        .map_err(|e| e.to_string())?;

    let valid = integrity::verify_integrity_data(&dir, &json_data).map_err(|e| e.to_string())?;
    if valid {
        println!("✅ Integrity check passed.");
    } else {
        println!("❌ Integrity check failed.");
        return Err("integrity check failed".to_string());
    }
    Ok(())
}

async fn handle_tests(tests: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        tests::run_tests(grind, tests).await;
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn handle_bundle(profile: Option<String>) {
    if let Some(grind) = util::parse_grind_file() {
        // need to compile classes first
        let build_flags = if let Some(profile) = profile {
            self::get_flags(&grind, profile)
        } else {
            String::new()
        };

        util::shell("rm -rf build && mkdir build");

        build::execute_build(&grind, BuildTarget::BuildOnly, build_flags);

        let _ = uberjar::build_fat_jar(&uberjar::FatJarConfig {
            output_jar: Path::new(&format!("build/{}.jar", grind.project.artifactId)),
            classes_dir: Path::new("target"),
            libs_dir: Path::new("libs"),
            group_id: &grind.project.groupId,
            artifact_id: &grind.project.artifactId,
            main_class: &format!("{}.{}", &grind.project.groupId, &grind.project.artifactId),
        });
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

pub fn get_flags(grind: &Grind, profile: String) -> String {
    let mut flags = String::new();

    if let Some(profiles) = &grind.project.profiles {
        if let Some(matched) = profiles.get(&profile) {
            if let Some(matched_flags) = &matched.flags {
                flags = matched_flags.join(" ")
            }
        }
    }
    flags
}

pub fn get_envs(grind: &Grind, profile: String) -> String {
    let mut envs = String::new();

    if let Some(profiles) = &grind.project.profiles {
        if let Some(matched) = profiles.get(&profile) {
            if let Some(matched_envs) = &matched.envs {
                envs = matched_envs
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<_>>()
                    .join(" ");
            }
        }
    }
    envs
}
