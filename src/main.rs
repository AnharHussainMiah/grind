use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use clap::{Parser, Subcommand};

mod config;

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


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => println!("todo -> {}", name),
        Commands::Build => println!("todo -> Build"),
        Commands::Run => println!("todo -> Run"),
        Commands::Add { dependencies } => println!("todo add deps -> {:#?}", dependencies),
        Commands::Remove { dependencies } => println!("todo remove deps -> {:#?}", dependencies),
    }
}

fn parse_grind_file() -> Option<Grind> {
    let grind_raw = fs::read_to_string("grind.yml").unwrap();
    let parsed: Grind = serde_yaml::from_str(&grind_raw).unwrap();
    Some(parsed)
}
