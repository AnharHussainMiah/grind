use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

/* -------------------------------------------------------------------------------------------------
The main struct defining the `grind` project configuration file. Changing this could potentially be
a *breaking* change.
NOTE: using serde(default) on Option<T> allows for missing fields
------------------------------------------------------------------------------------------------- */

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
    #[serde(default)]
    pub profiles: Option<HashMap<String, Profile>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub flags: Option<Vec<String>>,
    #[serde(default)]
    pub envs: Option<HashMap<String, String>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct Dependency {
    #[allow(non_snake_case)]
    pub groupId: String,
    #[allow(non_snake_case)]
    pub artifactId: String,
    pub version: String,
    #[serde(default)]
    pub scope: Option<String>,
}
