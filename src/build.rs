use crate::Grind;
use crate::util::ls_with_ext;
use crate::util::shell;
use std::fs;

#[derive(PartialEq)]
pub enum BuildTarget {
    BuildOnly,
    IncludeJar,
    IncludeTest,
}

pub fn execute_build(grind: &Grind, target: BuildTarget) {
    println!("==> 🔨 compiling project [{}]...", grind.project.artifactId);
    std::fs::create_dir_all(format!("{}/target", grind.project.artifactId)).unwrap();
    let out = shell("javac -d target -cp \"libs/*\" $(find src/main/java -name \"*.java\")");
    if !out.is_empty() {
        println!("{}", out);
    }
    if target == BuildTarget::IncludeJar {
        println!("==> 🔨 building manifest...");

        let external_jars = ls_with_ext("libs", "jar").unwrap_or_else(|err| {
            println!("⚠️ Error: unable to list external jars: {}", err);
            Vec::new()
        });
        let mut manifest = String::new();

        manifest.push_str(&format!(
            "Main-Class: {}.{}",
            grind.project.groupId, grind.project.artifactId
        ));

        if !external_jars.is_empty() {
            manifest.push_str(&format!("\nClass-Path: {}", external_jars.join("\n    ")));
        }
        manifest.push('\n');

        if fs::write("src/main/resources/manifest.mf", manifest).is_ok() {
            println!("{}", shell("rm -rf build/"));
            println!("{}", shell("mkdir -p build/"));
            let cmd = format!(
                "jar cfm build/{}.jar src/main/resources/manifest.mf -C target .",
                grind.project.artifactId
            );
            let out = shell(&cmd);
            if !out.is_empty() {
                println!("{}", out);
            }
        } else {
            println!("⚠️ Error: unbale to generate the manifest!");
        }
    }

    if target == BuildTarget::IncludeTest {
        println!(
            "==> 🔨 compiling tests for [{}]...",
            grind.project.artifactId
        );
        let out = shell(
            "javac -d target/test -cp \"target:libs/*\" $(find src/test/java -name \"*.java\")",
        );
        if !out.is_empty() {
            println!("{}", out);
        }
    }

    // clean up extra folders
    shell(&format!("rm -rf {}/", grind.project.artifactId));
}
