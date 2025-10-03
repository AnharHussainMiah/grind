use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

mod build;
mod config;
mod install;
mod mock;
mod scaffold;

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
    /// Initializes a new grind project structure
    Init {
        /// Name of the project to create
        name: String,
    },
    /// Install all the external libraries as defined in the grind.yml dependencies
    Install,
    /// Builds the project using the configuration in grind.yml
    Build,
    /// Runs the project (e.g., mvn spring-boot:run)
    Run,
    /// Adds a dependency to the project's grind.yml
    Add {
        /// List of dependencies to add (e.g., 'spring-boot postgresql-driver')
        dependencies: Vec<String>,
    },
    /// Removes a dependency from the project's grind.yml
    Remove {
        /// List of dependencies to remove
        dependencies: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => self::handle_init(&name),
        Commands::Build => println!("todo -> Build"),
        Commands::Install => self::install().await,
        Commands::Run => println!("todo -> Run"),
        Commands::Add { dependencies } => println!("todo add deps -> {:#?}", dependencies),
        Commands::Remove { dependencies } => println!("todo remove deps -> {:#?}", dependencies),
    }
}

fn handle_init(name: &str) {
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

async fn install() {
    if let Some(grind) = self::parse_grind_file() {
        install::handle_install(grind).await;
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
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute command");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
