use crate::Grind;
use crate::config::Dependency;
use crate::install;
use reqwest::Client;
use serde::Deserialize;
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
        // handle pinned version
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
                    "❌ WARNING: no match found for {}/{} {}",
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
    // resolve all deps to remove
    let resolved = install::resolve_all_deps(candidates).await;
    // remove them
    for dep in resolved {
        if let Err(e) = self::delete_jar(&dep) {
            println!("❌ Failed to download {:?}: {:?}", dep, e);
        }
    }
    // sync grind.yml
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
    //let group_path = dep.groupId.replace('.', "/");
    let artifact = &dep.artifactId;
    let version = &dep.version;

    let jar_name = format!("{}_{}_{}.jar", dep.groupId, artifact, version);
    let local_path = format!("libs/{}", jar_name);

    std::fs::remove_file(&local_path)?;

    println!("🗑️ REMOVED: {}", local_path);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SolrResponse {
    response: ResponseData,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ResponseData {
    numFound: u32,
    docs: Vec<Doc>,
}

#[derive(Debug, Deserialize, Clone)]
struct Doc {
    #[serde(rename = "g")]
    group_id: String,
    #[serde(rename = "a")]
    artifact_id: String,
    #[serde(rename = "v")]
    version: String,
}

async fn search_deps(group_id: &str, artifact: &str, version: &str) -> Option<Dependency> {
    let mut query = format!("g:{} AND a:{}", group_id, artifact);

    if !version.is_empty() {
        query = format!("{} AND v:\"{}\"", query, version);
    }

    // println!("DEBUG: {}", query);

    let url = format!(
        "https://search.maven.org/solrsearch/select?q={}&rows=1&core=gav&wt=json&sort=version+desc",
        urlencoding::encode(&query)
    );

    let client = Client::builder().user_agent("curl/8.5.0").build().unwrap();

    match client.get(url).send().await {
        Ok(response) => {
            let body = response.text().await;

            let j = match body {
                Ok(b) => b,
                Err(e) => {
                    println!("⚠️ ERROR: Unable to extract Reponse Body");
                    e.to_string()
                }
            };

            let parsed: SolrResponse = serde_json::from_str(&j).unwrap();

            if parsed.response.numFound > 1 {
                println!(
                    "✅ Match Found: {}/{} v{}",
                    &parsed.response.docs[0].group_id,
                    &parsed.response.docs[0].artifact_id,
                    &parsed.response.docs[0].version
                );
                let doc = parsed.response.docs[0].clone();

                return Some(Dependency {
                    groupId: doc.group_id,
                    artifactId: doc.artifact_id,
                    version: doc.version,
                    scope: Some("runtime".to_string()),
                });
            }
        }
        Err(e) => println!("⚠️ ERROR: {}", e),
    };

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
