use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

mod build;
mod config;
mod install;
mod mock;
mod run;
mod scaffold;
mod tasks;

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
           
        Grind hard, code harder v0.0.1
          - "builds, without the headache"
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
        /// Name of the project to create <NameSpace>/<ProjectName> e.g com.example/HelloWorld
        name: String,
    },
    /// Download all the external libraries and dependencies as defined in the grind.yml
    Install,
    /// Compile the project and builds a jar file.
    Build,
    /// Compile and run the project
    Run,
    /// Adds a dependency to the project's grind.yml
    Add {
        /// List of dependencies to add (e.g., 'spring-boot postgresql')
        dependencies: Vec<String>,
    },
    /// Removes a dependency from the project's grind.yml
    Remove {
        /// List of dependencies to remove
        dependencies: Vec<String>,
    },
    /// Run a custom task as defiend in the grind.yml, e.g grind task clean
    Task { job: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => self::handle_new(&name),
        Commands::Build => self::handle_build(),
        Commands::Install => self::handle_install().await,
        Commands::Run => self::handle_run(),
        Commands::Add { dependencies } => println!("todo add deps -> {:#?}", dependencies),
        Commands::Remove { dependencies } => println!("todo remove deps -> {:#?}", dependencies),
        Commands::Task { job } => self::handle_task(job),
    }
}

fn handle_new(name: &str) {
    if let Ok((namespace, artifact_id)) = self::parse_project_name(name) {
        let folder_path = Path::new(artifact_id);

        if !folder_path.exists() || !folder_path.is_dir() {
            scaffold::create(namespace, artifact_id);
        } else {
            println!(
                "Sorry project folder '{}' already exists, exiting...",
                artifact_id
            );
        }
    } else {
        println!(
            "Sorry '{}' is not a valid project name, requires a namespace and artifactId, e.g com.example/HelloWorld",
            name
        );
    }
}

fn handle_build() {
    if let Some(grind) = self::parse_grind_file() {
        build::execute_build(&grind, BuildTarget::IncludeJar);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

async fn handle_install() {
    if let Some(grind) = self::parse_grind_file() {
        install::execute_install(grind).await;
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn handle_run() {
    if let Some(grind) = self::parse_grind_file() {
        run::execute_run(grind);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn handle_task(job: String) {
    if let Some(grind) = self::parse_grind_file() {
        tasks::execute_task(grind, job);
    } else {
        println!("⚠️ Error: no grind.yml or invalid grind.yml")
    }
}

fn parse_project_name(input: &str) -> Result<(&str, &str), &'static str> {
    let mut parts = input.split('/');

    match (parts.next(), parts.next(), parts.next()) {
        (Some(first), Some(second), None) => Ok((first, second)),
        _ => Err("Input must contain exactly one '/' and two non-empty parts"),
    }
}

fn parse_grind_file() -> Option<Grind> {
    let grind_raw = fs::read_to_string("grind.yml").unwrap();
    let parsed: Grind = serde_yaml::from_str(&grind_raw).unwrap();
    Some(parsed)
}

fn shell(cmd: &str) -> String {
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

fn ls_with_ext(dir: &str, extension: &str) -> std::io::Result<Vec<String>> {
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
