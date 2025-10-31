use crate::Grind;
use crate::build;
use crate::util::shell_stream;
use crate::RunArgs;

use crate::BuildTarget;

pub fn execute_run(grind: Grind, args: &RunArgs) {

    build::execute_build(&grind, BuildTarget::BuildOnly, args.flags.to_string());
    println!("==> 🚀 running project [{}]...", grind.project.artifactId);

    let mut cmd = format!(
        "java -cp \"target:libs/*\" {}.{} {}",
        grind.project.groupId, grind.project.artifactId,
        args.args.join(" ")
    );

    if !args.envs.is_empty() {
        cmd = format!("{} {}", args.envs, cmd);
    }
    // println!("DEBUG: using cmd -> '{}'", cmd);
    let _ = shell_stream(&cmd);
}
