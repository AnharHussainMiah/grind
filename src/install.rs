use crate::Grind;
use crate::config::Dependency;
use regex::Regex;
use serde_xml_rs::from_str;
use std::collections::HashSet;
use std::collections::VecDeque;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::mock::FAKE_POM;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    dependencies: Option<Dependencies>,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    dependency: Vec<Dependency>,
}

pub async fn handle_install(grind: Grind) {
    let resolved = self::resolve_all_deps(grind.project.dependencies).await;
    for dep in resolved {
        if let Err(e) = self::download_jar(&dep).await {
            println!("==> Failed to download {:?}: {:?}", dep, e);
        }
    }
}

async fn resolve_all_deps(initial_deps: Vec<Dependency>) -> HashSet<Dependency> {
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
    let mut deps: Vec<Dependency> = Vec::new();

    let pom = self::get_pom(dep.clone()).await;
    if let Ok(mut extracted) = self::extract_dependencies(&pom) {
        for ex in &mut extracted {
            if ex.version.contains("$") {
                // println!("==> resolving version number for {}", ex.artifactId);
                let cleaned = &ex
                    .version
                    .replace("$", "")
                    .replace("{", "")
                    .replace("}", "");
                if let Some(resolved_version) = self::extract_xml_value(&pom, &cleaned) {
                    ex.version = resolved_version;
                }
            }
        }
        for ex in extracted {
            if ex.scope != Some("test".to_string()) || ex.scope == None {
                deps.push(ex);
            }
        }
    }
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

async fn get_pom(dep: Dependency) -> String {
    // return FAKE_POM.to_string();

    // check cache
    if let Ok(_) = tokio::fs::create_dir_all("cache").await {
        let pom_name = format!("{}_{}_{}.pom", dep.groupId, dep.artifactId, dep.version);
        let local_path = format!("cache/{}", pom_name);

        if Path::new(&local_path).exists() {
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
                tokio::fs::write(local_path, b.clone()).await.unwrap_or_else(|e| eprintln!("Failed to write file: {}", e));
                b
            },
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

    // let query = format!(r#"g:"{}" AND a:"{}""#, group, artifact);
    // let url = format!(
    //     "https://search.maven.org/solrsearch/select?q={}&rows=200&core=gav&wt=json",
    //     urlencoding::encode(&query)
    // );

#[derive(Debug)]
enum DownloadError {
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    Http(reqwest::StatusCode),
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::Reqwest(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::Io(err)
    }
}

async fn download_jar(dep: &Dependency) -> Result<(), DownloadError> {
    let group_path = dep.groupId.replace('.', "/");
    let artifact = &dep.artifactId;
    let version = &dep.version;

    // ✅ Flattened JAR filename
    let jar_name = format!("{}_{}_{}.jar", dep.groupId, artifact, version);
    let local_path = format!("lib/{}", jar_name);

    // ✅ Skip if already exists
    if Path::new(&local_path).exists() {
        println!("Already exists, skipping: {}", local_path);
        return Ok(());
    }

    // ✅ Ensure "lib" directory exists
    fs::create_dir_all("lib").await?;

    // ✅ Maven Central URL
    let url = format!(
        "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.jar",
        group_path, artifact, version, artifact, version
    );

    println!("📥 Downloading: {}", url);

    let resp = reqwest::get(&url).await?;

    if !resp.status().is_success() {
        return Err(DownloadError::Http(resp.status()));
    }

    let bytes = resp.bytes().await?;
    let mut file = fs::File::create(&local_path).await?;
    file.write_all(&bytes).await?;

    Ok(())
}
