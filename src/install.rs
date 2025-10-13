use crate::Grind;
use crate::config::Dependency;
use crate::pom::Pom;
use crate::lock;
use regex::Regex;
use semver::Version;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// use crate::mock::FAKE_POM;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    dependencies: Option<Dependencies>,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    dependency: Vec<Dependency>,
}

pub async fn execute_install(grind: Grind) {
    // first check the lock file
    if let Ok(locked) = lock::get_lock_file() {
        if grind.project.dependencies == locked.inputDeps {
            println!("✅ No dependency changes detected, using grind.lock...");
            for dep in locked.lockedDeps {
                if let Err(e) = self::download_jar(&dep).await {
                    println!("⚠️ Failed to download [{:?}]: {:?}", dep, e);
                }
            }
            return;
        }
    }
    println!("⚙️ need to resolve all dependencies...");
    let resolved = self::resolve_all_deps(grind.project.dependencies.clone()).await;
    let resolved = self::fix_collisions(resolved);
    for dep in &resolved {
        if let Err(e) = self::download_jar(&dep).await {
            println!("⚠️ Failed to download [{:?}]: {:?}", dep, e);
        }
    }
    // lock deps
    lock::lock_file(&grind.project.dependencies, &resolved.into_iter().collect());
}

pub async fn resolve_all_deps(initial_deps: Vec<Dependency>) -> HashSet<Dependency> {
    let mut resolved = HashSet::new();

    let mut to_visit = initial_deps
        .into_iter()
        .filter(|dep| dep.scope.as_deref() != Some("test"))
        .collect::<VecDeque<_>>();

    while let Some(dep) = to_visit.pop_front() {
        if resolved.contains(&dep) {
            continue;
        }

        resolved.insert(dep.clone());

        let transitive = self::fetch_deps(&dep).await;

        for new_dep in transitive {
            if !resolved.contains(&new_dep) {
                to_visit.push_back(new_dep);
            }
        }
    }
    resolved
}

async fn fetch_deps(dep: &Dependency) -> Vec<Dependency> {
    let deps: Vec<Dependency> = Vec::new();

    let raw = self::get_pom(dep.clone()).await;

    // we must compute the Effective POM to handle all parent <-> child, Imports, and properties

    if let Ok(pom) = from_str::<Pom>(&raw) {

    }


    // if let Ok(mut extracted) = self::extract_dependencies(&pom) {
    //     for ex in &mut extracted {
    //         if ex.version.contains("$") {
    //             // println!("==> resolving version number for {}", ex.artifactId);
    //             let cleaned = &ex
    //                 .version
    //                 .replace("$", "")
    //                 .replace("{", "")
    //                 .replace("}", "");
    //             if let Some(resolved_version) = self::extract_xml_value(&pom, &cleaned) {
    //                 ex.version = resolved_version;
    //             }
    //         }
    //     }
    //     for ex in extracted {
    //         if ex.scope != Some("test".to_string()) || ex.scope == None {
    //             deps.push(ex);
    //         }
    //     }
    // }
    deps
}

fn extract_dependencies(xml: &str) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let project: Project = from_str(xml)?;

    let deps = project
        .dependencies
        .unwrap_or(Dependencies { dependency: vec![] })
        .dependency
        .into_iter()
        .collect();

    Ok(deps)
}

pub async fn get_pom(dep: Dependency) -> String {
    // return FAKE_POM.to_string();

    // check cache
    if let Ok(_) = tokio::fs::create_dir_all("cache").await {
        let pom_name = format!("{}_{}_{}.pom", dep.groupId, dep.artifactId, dep.version);
        let local_path = format!("cache/{}", pom_name);

        if Path::new(&local_path).exists() {
            // TODO: compute the POM's md5 and compare with remote md5 only
            // use the cache if the remote file has not changed.
            if let Ok(cached) = tokio::fs::read_to_string(local_path).await {
                return cached;
            }
        }
    }

    println!("==> fetching POM.xml for {}", dep.artifactId);
    if let Ok(response) = reqwest::get(self::build_pom_url(
        &dep.groupId,
        &dep.artifactId,
        &dep.version,
    ))
    .await
    {
        let body = response.text().await;

        return match body {
            Ok(b) => {
                // cache POM
                let pom_name = format!("{}_{}_{}.pom", dep.groupId, dep.artifactId, dep.version);
                let local_path = format!("cache/{}", pom_name);
                tokio::fs::write(local_path, b.clone())
                    .await
                    .unwrap_or_else(|e| eprintln!("Failed to write file: {}", e));
                b
            }
            Err(e) => e.to_string(),
        };
    }
    "error!".to_string()
}

fn extract_xml_value(xml: &str, element: &str) -> Option<String> {
    let pattern = format!(r"<{e}[^>]*>(.*?)</{e}>", e = regex::escape(element));
    let re = Regex::new(&pattern).ok()?;
    re.captures(xml)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn build_pom_url(group: &str, artifact: &str, version: &str) -> String {
    let gpath = group.replace(".", "/");
    format!(
        "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.pom",
        gpath, artifact, version, artifact, version
    )
}

async fn download_jar(dep: &Dependency) -> Result<(), String> {
    let group_path = dep.groupId.replace('.', "/");
    let artifact = &dep.artifactId;
    let version = &dep.version;

    // ✅ Flattened JAR filename
    let jar_name = format!("{}_{}_{}.jar", dep.groupId, artifact, version);
    let local_path = format!("libs/{}", jar_name);

    // ✅ Skip if already exists
    if Path::new(&local_path).exists() {
        println!("Already exists, skipping: {}", local_path);
        return Ok(());
    }

    // ✅ Ensure "lib" directory exists
    fs::create_dir_all("libs")
        .await
        .map_err(|e| e.to_string())?;

    // ✅ Maven Central URL
    let url = format!(
        "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.jar",
        group_path, artifact, version, artifact, version
    );

    println!("📥 Downloading: {}", url);

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "Unable to download, HTTP Status Code: {}",
            resp.status()
        ));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&local_path)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;

    Ok(())
}

fn fix_collisions(deps: HashSet<Dependency>) -> HashSet<Dependency> {
    /* ---------------------------------------------------------------------------------------------
    modern build tools and including the latest versions of maven use the "newest" version wins
    when it comes to dependency "collisions" (different versions of the same artifact).

    It is better to go with the newest otherwise one could regress and introduce bugs etc.

    However this needs to be combined with a "lock" file to "freeze" the fully resolved tree, this
    makes it deterministic and reproducible.
    --------------------------------------------------------------------------------------------- */
    let mut latest_versions: HashMap<(String, String), Dependency> = HashMap::new();

    for dep in deps {
        let key = (dep.groupId.clone(), dep.artifactId.clone());

        latest_versions
            .entry(key)
            .and_modify(|existing| {
                if self::is_version_newer(&existing.version, &dep.version) {
                    *existing = dep.clone();
                }
            })
            .or_insert(dep);
    }

    latest_versions.into_values().collect()
}

fn is_version_newer(source: &str, target: &str) -> bool {
    // source < targer
    if let (Ok(s), Ok(t)) = (Version::parse(source), Version::parse(target)) {
        return s < t;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Dependency;
    // use semver::Version;

    #[test]
    fn test_deduplicate_dependencies() {
        let mut deps = HashSet::new();

        deps.insert(Dependency {
            groupId: "com.example".to_string(),
            artifactId: "lib1".to_string(),
            version: String::from("1.0.0"),
            scope: None,
        });

        deps.insert(Dependency {
            groupId: "com.example".to_string(),
            artifactId: "lib1".to_string(),
            version: String::from("2.0.0"), // Should be kept
            scope: None,
        });

        deps.insert(Dependency {
            groupId: "com.example".to_string(),
            artifactId: "lib2".to_string(),
            version: String::from("0.9.1"), // Only one, should be kept
            scope: None,
        });

        deps.insert(Dependency {
            groupId: "org.other".to_string(),
            artifactId: "lib3".to_string(),
            version: String::from("3.1.4"),
            scope: None,
        });

        deps.insert(Dependency {
            groupId: "org.other".to_string(),
            artifactId: "lib3".to_string(),
            version: String::from("3.2.0"), // Should be kept
            scope: None,
        });

        let result = fix_collisions(deps);

        // Should contain only 3 dependencies: latest of lib1, lib2, and lib3
        assert_eq!(result.len(), 3);

        // Check that correct versions are picked
        let expected = vec![
            Dependency {
                groupId: "com.example".to_string(),
                artifactId: "lib1".to_string(),
                version: String::from("2.0.0"),
                scope: None,
            },
            Dependency {
                groupId: "com.example".to_string(),
                artifactId: "lib2".to_string(),
                version: String::from("0.9.1"),
                scope: None,
            },
            Dependency {
                groupId: "org.other".to_string(),
                artifactId: "lib3".to_string(),
                version: String::from("3.2.0"),
                scope: None,
            },
        ];

        for dep in expected {
            assert!(
                result.contains(&dep),
                "Missing expected dependency: {:?}",
                dep
            );
        }
    }
}
