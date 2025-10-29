use crate::Grind;
use crate::build;
use crate::get_envs;
use crate::get_flags;
use crate::util::shell_stream;

use crate::BuildTarget;

pub fn execute_run(grind: Grind, profile: Option<String>) {
    let mut flags = String::new();
    let mut envs = String::new();

    if let Some(profile) = profile {
        flags = get_flags(&grind, profile.clone());
        envs = get_envs(&grind, profile);
    }

    build::execute_build(&grind, BuildTarget::BuildOnly, flags);
    println!("==> 🚀 running project [{}]...", grind.project.artifactId);

    let mut cmd = format!(
        "java -cp \"target:libs/*\" {}.{}",
        grind.project.groupId, grind.project.artifactId
    );

    if !envs.is_empty() {
        cmd = format!("{} {}", envs, cmd);
    }

    // println!("DEBUG: using cmd -> '{}'", cmd);
    let _ = shell_stream(&cmd);
    // if !out.is_empty() {
    //     println!("{}", out);
    // }
}
