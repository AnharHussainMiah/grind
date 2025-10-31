use crate::config;
use crate::install;
use quick_xml::de::Deserializer as XmlDeserializer;
use serde::Deserialize;
use serde_path_to_error as path;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PomId {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl fmt::Display for PomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: Option<String>,
}

pub async fn get_effective_dependencies(
    root_pom_id: PomId,
    visited: &mut HashSet<PomId>,
) -> Option<Vec<EffectiveDependency>> {
    match resolve_context_recursive(&root_pom_id, visited).await {
        Ok((pom, context)) => {
            // If context resolution is successful, calculate the final dependencies.
            let mut effective_deps = Vec::new();
            for dep in pom.dependencies.dependency {
                let dep_key = format!("{}:{}", dep.group_id, dep.artifact_id);
                // println!("WALKING -> {}", dep_key);
                let version = dep.version.as_deref().or_else(|| {
                    context
                        .dependency_management
                        .get(&dep_key)
                        .and_then(|d| d.version.as_deref())
                });

                if let Some(v) = version {
                    // println!("MERGE SCOPE: [{:?}] {:?}", dep, dep.scope);
                    let final_dep = EffectiveDependency {
                        group_id: substitute_properties(&dep.group_id, &context.properties),
                        artifact_id: substitute_properties(&dep.artifact_id, &context.properties),
                        version: substitute_properties(v, &context.properties),
                        scope: dep.scope.clone().or(Some("compile".to_string())),
                    };

                    let mut is_optional = false;
                    if let Some(optional) = dep.optional {
                        if optional.contains("true") {
                            is_optional = true;
                        }
                    }

                    if !is_optional {
                        effective_deps.push(final_dep);
                    }
                }
                // Dependencies without a concrete version are ignored.
            }
            Some(effective_deps)
        }
        Err(e) => {
            eprintln!("Failed to resolve dependencies for {}: {}", root_pom_id, e);
            None
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ResolutionContext {
    dependency_management: HashMap<String, Dependency>,
    properties: HashMap<String, String>,
}

async fn resolve_context_recursive(
    pom_id: &PomId,
    visited: &mut HashSet<PomId>,
) -> Result<(Pom, ResolutionContext), String> {
    if visited.contains(pom_id) {
        let pom_xml = install::get_pom(config::Dependency {
            groupId: pom_id.group_id.clone(),
            artifactId: pom_id.artifact_id.clone(),
            version: pom_id.version.clone(),
            scope: Some("compile".to_string()),
        })
        .await;

        // println!("DEBUG: Parsing POM {:?}", pom_id);
        let pom = parse_pom_from_str(&pom_xml)?;
        println!("Cyclic dependency detected: {}", pom_id);
        return Ok((pom, ResolutionContext::default()));
    }

    visited.insert(pom_id.clone());

    let pom_xml = install::get_pom(config::Dependency {
        groupId: pom_id.group_id.clone(),
        artifactId: pom_id.artifact_id.clone(),
        version: pom_id.version.clone(),
        scope: Some("compile".to_string()),
    })
    .await;

    // println!("DEBUG: Parsing POM {:?}", pom_id);
    let mut pom = parse_pom_from_str(&pom_xml)?;

    // Parent Resolution "Walk Up"
    let (mut context, parent_pom) = if let Some(parent) = &pom.parent {
        let parent_id = PomId {
            group_id: parent.group_id.clone(),
            artifact_id: parent.artifact_id.clone(),
            version: parent.version.clone(),
        };

        let (parent_pom, parent_context) =
            Box::pin(resolve_context_recursive(&parent_id, visited)).await?;
        (parent_context, Some(parent_pom))
    } else {
        (ResolutionContext::default(), None)
    };

    // Inherit groupId and version from parent if not specified.
    if pom.group_id.is_none() {
        pom.group_id = pom.parent.as_ref().map(|p| p.group_id.clone());
    }
    if pom.version.is_none() {
        pom.version = pom.parent.as_ref().map(|p| p.version.clone());
    }

    // Merge properties "Walk Down"
    // Add project-specific properties. These can be used to resolve versions.
    let mut current_properties = HashMap::new();
    if let Some(gid) = &pom.group_id {
        current_properties.insert("project.groupId".to_string(), gid.clone());
    }
    if let Some(aid) = &pom.artifact_id {
        current_properties.insert("project.artifactId".to_string(), aid.clone());
    }
    if let Some(ver) = &pom.version {
        current_properties.insert("project.version".to_string(), ver.clone());
    }

    // NOTE: (order or precedence): parent properties are applied first
    if let Some(p_pom) = parent_pom {
        if let Some(p_gid) = p_pom.group_id {
            current_properties.insert("project.parent.groupId".to_string(), p_gid);
        }
        if let Some(p_ver) = p_pom.version {
            current_properties.insert("project.parent.version".to_string(), p_ver);
        }
    }

    // NOTE: (order or precedence): Child properties overwrite parent properties.
    context.properties.extend(current_properties);
    context.properties.extend(pom.properties.clone());

    for dep in pom.dependency_management.dependencies.dependency.iter() {
        let key = format!("{}:{}", dep.group_id, dep.artifact_id);
        // println!("DEBUG: key {:?} | {:?}", key, dep.scope);
        // Child `dependencyManagement` takes precedence.
        context
            .dependency_management
            .entry(key)
            .or_insert_with(|| dep.clone());
    }

    // BOM resolution "Walk sideways"
    let managed_deps_clone = context
        .dependency_management
        .values()
        .cloned()
        .collect::<Vec<_>>();
    for dep in managed_deps_clone {
        if dep.scope.as_deref() == Some("import") && dep.r#type.as_deref() == Some("pom") {
            // println!("DEBUG-IMPORT: {:?} | {:?}", dep, pom_id);
            let version = substitute_properties(dep.version.as_ref().unwrap(), &context.properties);
            let import_pom_id = PomId {
                group_id: substitute_properties(&dep.group_id, &context.properties),
                artifact_id: substitute_properties(&dep.artifact_id, &context.properties),
                version,
            };

            let (_, import_context) =
                Box::pin(resolve_context_recursive(&import_pom_id, visited)).await?;

            // NOTE: existing enteries must be preserved
            for (key, val) in import_context.dependency_management {
                context.dependency_management.entry(key).or_insert(val);
            }
        }
    }

    Ok((pom, context))
}

fn parse_pom_from_str(pom_xml: &str) -> Result<Pom, String> {
    let mut deserializer = XmlDeserializer::from_str(pom_xml);
    let result: Result<Pom, _> = path::deserialize(&mut deserializer);
    result.map_err(|e| format!("Error at {}: {}", e.path(), e))
}

fn substitute_properties(value: &str, properties: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    for (key, val) in properties {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, val);
    }
    result
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Pom {
    group_id: Option<String>,
    artifact_id: Option<String>,
    version: Option<String>,
    parent: Option<Parent>,
    #[serde(default)]
    properties: Properties,
    #[serde(default)]
    dependency_management: DependencyManagement,
    #[serde(default)]
    dependencies: Dependencies,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Parent {
    group_id: String,
    artifact_id: String,
    version: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(transparent)]
struct Properties(HashMap<String, String>);

impl Deref for Properties {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Properties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Properties {
    type Item = (String, String);
    type IntoIter = <HashMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Properties {
    type Item = (&'a String, &'a String);
    type IntoIter = <&'a HashMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Properties {
    type Item = (&'a String, &'a mut String);
    type IntoIter = <&'a mut HashMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct DependencyManagement {
    #[serde(default)]
    dependencies: Dependencies,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct Dependencies {
    #[serde(rename = "dependency", default)]
    dependency: Vec<Dependency>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Dependency {
    group_id: String,
    artifact_id: String,
    version: Option<String>,
    #[serde(rename = "type")]
    r#type: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    optional: Option<String>,
}
