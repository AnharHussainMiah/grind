use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Grind {
    pub project: Project,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub groupId: String,
    pub artifactId: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub dependencies: Vec<Dependency>,
    pub tasks: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub groupId: String,
    pub artifactId: String,
    pub version: String,
    #[serde(default)]  // Optional field: default to None if missing
    pub scope: Option<String>,
}