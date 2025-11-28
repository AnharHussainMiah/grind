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
mod java;
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
use crate::util::shell;

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
                    v0.8.0
"#;

const LICENSE: &str = r#"
Licensed under the GNU GPLv3 or later.

See <https://www.gnu.org/licenses/gpl-3.0.html> for details.
© 2025 Anhar Hussain Miah <anharhussainmiah@googlemail.com>"
"#;

#[derive(Parser, Debug)]
#[command(author = "Anhar Miah", version, about = LOGO, long_about = None, after_help = LICENSE)]
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
        profile: Vec<String>,
    },
    /// Compile and run the project
    Run {
        /// the defined profile to run with, these include compiler flags, and environment variables
        profile: Vec<String>,
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
        profile: Vec<String>,
    },
    /// Manage Java Versions
    Java {
        #[command(subcommand)]
        java: JavaVersionManger,
    },
}

#[derive(Subcommand, Debug)]
enum JavaVersionManger {
    /// List all available JDKs
    List,
    /// List the current JDK in use
    Current,
    /// This will automatically set the version of the JDK (over riding the system) and downloading if required
    Use {
        /// the specific version to install and setup
        version: String,
    },
    /// Remove any installed JDK (this does not affect the system wide JDK)
    Remove,
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
        Commands::Bundle { profile } => self::handle_bundle(profile),
        Commands::Java { java } => match java {
            JavaVersionManger::List => java::list().await,
            JavaVersionManger::Current => java::current(),
            JavaVersionManger::Use { version } => java::_use(version).await,
            JavaVersionManger::Remove => java::remove(),
        },
    }
}

fn handle_new(name: &str) {
    match self::parse_project_name(name) {
        Ok((namespace, artifact_id)) => {
            let folder_path = Path::new(artifact_id);

            if !folder_path.exists() || !folder_path.is_dir() {
                scaffold::create(namespace, artifact_id);
            } else {
                println!(
                    "⚠️ Sorry project folder '{}' already exists, exiting...",
                    artifact_id
                );
            }
        }
        Err(e) => {
            println!(
                "⚠️ Sorry '{}' is not a valid project name, requires a namespace and artifactId, e.g com.example/HelloWorld",
                name
            );
            println!("{e}");
        }
    }
}

fn handle_build(profile: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        let args = self::get_run_args(&grind, profile);

        build::execute_build(&grind, BuildTarget::IncludeJar, args.flags);
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

fn handle_run(profile: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        let args = self::get_run_args(&grind, profile);

        run::execute_run(grind, &args);
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

    let (namespace, artifact_id) = match (parts.next(), parts.next(), parts.next()) {
        (Some(first), Some(second), None) => (first, second),
        _ => return Err("⚠️ Input must contain exactly one '/' and two non-empty parts"),
    };

    Ok((
        validate_namespace(namespace)?,
        validate_artifact_id(artifact_id)?,
    ))
}

fn is_valid_java_identifier(identifier: &str) -> bool {
    // Make sure we have at least one character
    if identifier.is_empty() {
        return false;
    }

    // Check valid characters
    for c in identifier.chars() {
        match c {
            '_' | '$' | 'A'..='Z' | 'a'..='z' | '0'..='9' => continue,
            _ => return false,
        }
    }

    // Ensure we don't start with a digit character.
    match identifier.chars().nth(0).unwrap() {
        '0'..='9' => return false,
        _ => {}
    }

    true
}

fn validate_namespace(namespace: &str) -> Result<&str, &'static str> {
    for part in namespace.split('.') {
        match is_valid_java_identifier(part) {
            true => continue,
            false => return Err("⚠️ Your namespace contains an invalid java identifier"),
        }
    }

    Ok(namespace)
}

fn validate_artifact_id(artifact_id: &str) -> Result<&str, &'static str> {
    match is_valid_java_identifier(artifact_id) {
        true => Ok(artifact_id),
        false => Err("⚠️ Your artifactId contains an invalid java identifier"),
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

fn handle_bundle(profile: Vec<String>) {
    if let Some(grind) = util::parse_grind_file() {
        // need to compile classes first

        let args = self::get_run_args(&grind, profile);

        build::execute_build(&grind, BuildTarget::BuildOnly, args.flags);

        shell("rm -rf build/ && mkdir build");

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

struct RunArgs {
    flags: String,
    envs: String,
    args: Vec<String>,
}

fn get_run_args(grind: &Grind, args: Vec<String>) -> RunArgs {
    /* ---------------------------------------------------------------------------------------------
    because we have a list of arguments provided, there is some overlap in the sense the first
    argument is ambiguous, because it could mean the "profile" OR it could just be the first normal
    argument. This function takes the first argument and sees if any profiles match. If there is a
    match we simply "consume" it from the list and return the flags + envs + args e.g:

    -> profile:
            \_ flags
            \_ envs

    -> rest of args
    --------------------------------------------------------------------------------------------- */
    let mut xargs = args.clone();
    let mut flags = String::new();
    let mut envs = String::new();

    if let Some(profile) = xargs.first() {
        flags = self::get_flags(&grind, profile.to_string());
        envs = self::get_envs(&grind, profile.to_string());

        if !flags.is_empty() || !envs.is_empty() {
            xargs.remove(0);
        }
    }

    RunArgs {
        flags: flags,
        envs: envs,
        args: xargs,
    }
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
