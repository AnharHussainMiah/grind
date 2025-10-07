use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Grind {
    pub project: Project,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    #[allow(non_snake_case)]
    pub groupId: String,
    #[allow(non_snake_case)]
    pub artifactId: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub dependencies: Vec<Dependency>,
    pub tasks: HashMap<String, String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct Dependency {
    #[allow(non_snake_case)]
    pub groupId: String,
    #[allow(non_snake_case)]
    pub artifactId: String,
    pub version: String,
    #[serde(default)] // Optional field: default to None if missing
    pub scope: Option<String>,
}
