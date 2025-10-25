use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Metadata {
    groupId: String,
    artifactId: String,
    versioning: Versioning,
}

#[derive(Debug, Deserialize)]
struct Versioning {
    release: Option<String>,
    versions: Versions,
}

#[derive(Debug, Deserialize)]
struct Versions {
    version: Vec<String>,
}

pub async fn fetch_maven_metadata(
    group_id: &str,
    artifact_id: &str,
) -> Result<(Option<String>, Vec<String>), String> {
    // Convert groupId into Maven repo path
    let group_path = group_id.replace('.', "/");
    let url = format!(
        "https://repo1.maven.org/maven2/{}/{}/maven-metadata.xml",
        group_path, artifact_id
    );

    println!("🌎 Fetching metadata from: {}", url);

    // Fetch the metadata XML
    let xml_data = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // Parse XML using serde_xml_rs
    let metadata: Metadata = serde_xml_rs::from_str(&xml_data).map_err(|e| e.to_string())?;

    let release = metadata.versioning.release;
    let versions = metadata.versioning.versions.version;

    Ok((release, versions))
}
