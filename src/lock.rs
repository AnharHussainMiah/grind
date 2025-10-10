use crate::config::Dependency;
use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Lock {
    pub inputDeps: Vec<Dependency>,
    pub lockedDeps: Vec<Dependency>,
}

pub fn get_lock_file() -> Result<Lock, String> {
    let lock_raw = fs::read_to_string("grind.lock").map_err(|e| e.to_string())?;
    let parsed: Lock = serde_yaml::from_str(&lock_raw).map_err(|e| e.to_string())?;
    Ok(parsed)
}

pub fn lock_file(input_deps: &Vec<Dependency>, locked_deps: &Vec<Dependency>) {
    let lock = Lock {
        inputDeps: input_deps.to_vec(),
        lockedDeps: locked_deps.to_vec(),
    };

    if let Ok(updated) = serde_yaml::to_string(&lock) {
        if let Ok(_) = fs::write("grind.lock", updated) {
            println!("🔃 grind.lock synced..");
        }
    }
}
