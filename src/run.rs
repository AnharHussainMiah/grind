use crate::Grind;
use crate::build;
use crate::shell;

use crate::BuildTarget;

pub fn execute_run(grind: Grind) {
    build::execute_build(&grind, BuildTarget::BuildOnly);
    println!("==> 🚀 running project [{}]...", grind.project.artifactId);
    let cmd = format!(
        "java -cp \"target:libs/*\" {}",
        grind.project.artifactId
    );

    let out = shell(&cmd);
    if !out.is_empty() {
        println!("{}", out);
    }
}
