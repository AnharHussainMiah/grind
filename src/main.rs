use clap::{Parser, Subcommand};
use std::path::Path;

mod build;
mod config;
mod install;
mod lock;
mod manage;
mod mock;
mod pom;
mod run;
mod scaffold;
mod tasks;
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

        - "builds, without the headache"
                    v0.4.0
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
        deps: Vec<String>,
    },
    /// Removes a dependency from the project's grind.yml
    Remove {
        /// List of dependencies to remove
        deps: Vec<String>,
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
        Commands::Add { deps } => self::handle_add(deps).await,
        Commands::Remove { deps } => self::handle_remove(deps).await,
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
    if let Some(grind) = util::parse_grind_file() {
        build::execute_build(&grind, BuildTarget::IncludeJar);
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

fn handle_run() {
    if let Some(grind) = util::parse_grind_file() {
        run::execute_run(grind);
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
        _ => Err("Input must contain exactly one '/' and two non-empty parts"),
    }
}
