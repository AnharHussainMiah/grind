
use serde::Deserialize;

use crate::install;
use std::collections::HashSet;
use std::collections::HashMap;
use serde_xml_rs::from_str;
use crate::config::Dependency;


#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct Pom {
    pub groupId: String,
    pub artifactId: String,
    pub version: String,
    pub packaging: String,
    #[serde(default)]
    pub parent: Option<Parent>,
    #[serde(default)]
    pub properties: Option<HashMap<String, String>>,
    #[serde(default)]
    pub dependencyManagement: Option<Dependencies>,
    #[serde(default)]
    pub dependencies: Option<Dependencies>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dependencies {
    dependencies: Option<Vec<PomDependency>>
}


#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct Parent {
    pub groupId: String,
    pub artifactId: String,
    pub version: String,
    #[serde(default)]
    pub relativePath: Option<String>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct PomDependency {
    pub groupId: String,
    pub artifactId: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub optional: Option<String>
}

pub async fn compute_effective_pom(mut base: Pom) -> Pom {
    let mut parent_stack: Vec<Pom> = Vec::new();
    let mut current = base.clone();

    // 1. resolve all parents
    while let Some(pom_ref) = &current.parent {
        if let Ok(parent_pom) = self::extract_pom(&pom_ref.groupId, &pom_ref.artifactId, &pom_ref.version).await {
            parent_stack.push(parent_pom.clone());
            current = parent_pom;
        }
    }

    // 2. Merge parent chaind down
    while let Some(parent) = parent_stack.pop() {
        base = self::merge_parent_into_child(parent, base);
    }

    let mut import_stack: Vec<Pom> = Vec::new();
    let mut seen_imports: HashSet<String> = HashSet::new(); // Prevent cycles

    // 3. handle dependencyMangement imports
    if let Some(dm) = &base.dependencyManagement {
        for dep in &dm.dependencies {
            if dep.scope.as_deref() == Some("import") && dep.type_.as_deref() == Some("pom") {
                let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                if seen_imports.insert(key.clone()) {
                    if let Ok(import_pom) = self::extract_pom(
                        &dep.group_id,
                        &dep.artifact_id,
                        dep.version.as_ref().expect("Import without version!"),
                    ).await {
                        import_stack.push(import_pom);
                    }
                }
            }
        }
    }

    while let Some(imported_pom) = import_stack.pop() {
        base = self::merge_dependency_management(imported_pom, base);
    }

    // Step 4: Apply properties
    base = self::resolve_properties(base);

    base
}

fn merge_parent_into_child(parent: Pom, mut child: Pom) -> Pom {
    // Inherit groupId/version/packaging/dependencyManagement/etc.
    if child.groupId.is_none() {
        child.groupId = parent.groupId;
    }
    if child.version.is_none() {
        child.version = parent.version;
    }

    // Merge dependencyManagement
    child = merge_dependency_management(parent, child);

    // Merge properties
    child.properties = merge_maps(parent.properties, child.properties);

    // ...other merging logic...
    child
}

fn merge_dependency_management(parent: Pom, mut child: Pom) -> Pom {
    if let Some(parent_dm) = parent.dependencyManagement {
        child.dependencyManagement.get_or_insert(vec![]).dependencies.extend(parent_dm.dependencies);
    }
    child
}

fn resolve_properties(mut pom: Pom) -> Pom {
    let props = pom.properties.clone();
    let re = regex::Regex::new(r"\$\{(.+?)\}").unwrap();

    for (k, v) in pom.properties.iter_mut() {
        let resolved = re.replace_all(v, |caps: &regex::Captures| {
            props.get(&caps[1]).cloned().unwrap_or_else(|| caps[0].to_string())
        });
        *v = resolved.to_string();
    }

    // Apply to other fields if needed...
    pom
}


async fn extract_pom(group_id: &str, artifact_id: &str, version: &str) -> Result<Pom, String> {
    let raw = install::get_pom(Dependency {

    }).await;
    let pom: Pom = from_str(&raw).map_err(|e| e.to_string())?;
    Ok(pom)
}