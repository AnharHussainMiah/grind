use crate::Grind;
use crate::config::Dependency;
use crate::install;
use crate::metadata;
use std::fs;

pub async fn execute_add(grind: Grind, deps: Vec<String>) {
    let mut candidates = Vec::new();

    for dep in deps {
        let mut group_id = String::new();
        let mut version = String::new();
        let mut artifact = String::new();

        if dep.contains("/") {
            let tokens: Vec<&str> = dep.split("/").collect();
            if tokens.len() >= 2 {
                group_id = tokens[0].to_string();
                artifact = tokens[1].to_string();
            }
        }

        if artifact.contains("@") {
            if let Some((a, v)) = artifact.split_once("@") {
                let a = a.to_string();
                let v = v.to_string();
                artifact = a;
                version = v;
            }
        }

        let results = self::search_deps(&group_id, &artifact, &version).await;

        match results {
            Some(matched_dep) => candidates.push(matched_dep),
            None => {
                println!(
                    "❌ WARNING: no match found for {}/{} v{}",
                    group_id, artifact, version
                );
            }
        }
    }
    // now that we have a list of resolved candidates lets sync the grind.yml and install
    if !candidates.is_empty() {
        self::update_grind(grind, candidates).await;
    }
}

pub async fn execute_remove(mut grind: Grind, deps: Vec<String>) {
    let mut candidates = Vec::new();

    for dep in deps {
        let mut group_id = String::new();
        let mut artifact = String::new();

        if dep.contains("/") {
            let tokens: Vec<&str> = dep.split("/").collect();
            if tokens.len() >= 2 {
                group_id = tokens[0].to_string();
                artifact = tokens[1].to_string();
            }
        }

        if let Some(index) = grind
            .project
            .dependencies
            .iter()
            .position(|x| x.groupId == group_id && x.artifactId == artifact)
        {
            println!("⚙️ preparing to remove {} {}", group_id, artifact);
            candidates.push(grind.project.dependencies[index].clone());
            grind.project.dependencies.remove(index);
        } else {
            println!("❌ WARNING: no match found for {}/{}", group_id, artifact);
        }
    }

    let resolved = install::resolve_all_deps(candidates).await;

    for dep in resolved {
        if let Err(e) = self::delete_jar(&dep) {
            println!("❌ Failed to download {:?}: {:?}", dep, e);
        }
    }

    if let Ok(updated) = serde_yaml::to_string(&grind) {
        if fs::write("grind.yml", updated).is_ok() {
            println!("🔃 grind.yml synced..");
            // run install again
            install::execute_install(grind).await;
        }
    } else {
        println!("⚠️ Unable to sync grind.yml!")
    }
}

fn delete_jar(dep: &Dependency) -> Result<(), std::io::Error> {
    let artifact = &dep.artifactId;
    let version = &dep.version;

    let jar_name = format!("{}_{}_{}.jar", dep.groupId, artifact, version);
    let local_path = format!("libs/{}", jar_name);

    std::fs::remove_file(&local_path)?;

    println!("🗑️ REMOVED: {}", local_path);

    Ok(())
}

async fn search_deps(group_id: &str, artifact: &str, version: &str) -> Option<Dependency> {
    if let Ok((release, versions)) = metadata::fetch_maven_metadata(group_id, artifact).await {
        if !version.is_empty() {
            if let Some(matched) = versions.into_iter().find(|v| *v == version) {
                println!("✅ Match Found: {}/{} v{}", &group_id, &artifact, &matched);

                return Some(Dependency {
                    groupId: group_id.to_string(),
                    artifactId: artifact.to_string(),
                    version: matched,
                    scope: Some("compile".to_string()),
                });
            } else {
                return None;
            }
        }

        if let Some(v) = release {
            println!("✅ Match Found: {}/{} v{}", &group_id, &artifact, &v);

            return Some(Dependency {
                groupId: group_id.to_string(),
                artifactId: artifact.to_string(),
                version: v,
                scope: Some("compile".to_string()),
            });
        }
    }
    None
}

async fn update_grind(mut grind: Grind, candidates: Vec<Dependency>) {
    for dep in candidates {
        if !grind.project.dependencies.contains(&dep) {
            grind.project.dependencies.push(dep.clone());
        }
    }

    if let Ok(updated) = serde_yaml::to_string(&grind) {
        if fs::write("grind.yml", updated).is_ok() {
            println!("🔃 grind.yml synced..");

            install::execute_install(grind).await;
        }
    } else {
        println!("⚠️ Unable to sync grind.yml!")
    }
}
