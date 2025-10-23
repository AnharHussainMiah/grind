use crate::Grind;
use crate::config::Dependency;
use crate::lock;
use crate::pom;
use crate::pom::PomId;
use semver::Version;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// use crate::mock::FAKE_POM;

pub async fn execute_install(grind: Grind) {
    // first check the lock file
    if let Ok(locked) = lock::get_lock_file()
        && grind.project.dependencies == locked.inputDeps
    {
        println!("✅ No dependency changes detected, using grind.lock...");
        for dep in locked.lockedDeps {
            if let Err(e) = self::download_jar(&dep).await {
                println!("⚠️ Failed to download [{:?}]: {:?}", dep, e);
            }
        }
        return;
    }
    println!("⚙️ need to resolve all dependencies...");
    let resolved = self::resolve_all_deps(grind.project.dependencies.clone()).await;
    let resolved = self::fix_collisions(resolved);
    for dep in &resolved {
        if let Err(e) = self::download_jar(dep).await {
            /*
                we need to think about the "global" state, while we have fixed the "local"
                collisions, we may still be introducing an older version from a different dependency
                path.

                A failure could happen because of:
                
                    * (A) Download Issue (network) tempory blip
                    * (B) It's not a real JAR, e.g ${version} or [1.1.0,) etc
                    * (C) There is already a newer version

                if it's B or C then we need to remove the dep from the resolved list as we do NOT
                want to lock this into the lock file.

                When we have C, we need to also make sure the older version is physically deleted.

                Only A should go into the lock file, as in we should have that dep, even though it
                failed due to network issues.
            */
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

        // println!("DEBUG {:?}", transitive);

        for new_dep in transitive {
            if !resolved.contains(&new_dep) {
                to_visit.push_back(new_dep);
            }
        }
    }
    resolved
}

async fn fetch_deps(dep: &Dependency) -> Vec<Dependency> {
    let mut deps: Vec<Dependency> = Vec::new();

    let root_pom_id = PomId {
        group_id: dep.groupId.clone(),
        artifact_id: dep.artifactId.clone(),
        version: dep.version.clone(),
    };

    let mut visited = HashSet::new();

    println!("ℹ️ Resolving dependencies for {}...", root_pom_id);

    if let Some(rdeps) = pom::get_effective_dependencies(root_pom_id, &mut visited).await {
        println!("\nℹ️ Found {} effective dependencies:", rdeps.len());
        for rdep in rdeps {
            println!(
                "  - {}:{}:{} (Scope: {})",
                rdep.group_id,
                rdep.artifact_id,
                rdep.version,
                rdep.scope.as_deref().unwrap_or("compile")
            );

            if let Some(v) = &rdep.scope
                && (v.contains("compile") || v.contains("runtime"))
            {
                deps.push(Dependency {
                    groupId: rdep.group_id,
                    artifactId: rdep.artifact_id,
                    version: rdep.version,
                    scope: rdep.scope,
                })
            }
        }
    } else {
        println!("⚠️ Could not resolve dependencies.");
    }
    deps
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

    println!("🌎 ==> fetching POM.xml for {}", dep.artifactId);
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
                    .unwrap_or_else(|e| eprintln!("⚠️ Failed to write file: {}", e));
                b
            }
            Err(e) => e.to_string(),
        };
    }
    "error!".to_string()
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
        println!("📦 Already exists, skipping: {}", local_path);
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
            "⚠️ Unable to download, HTTP Status Code: {}",
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
