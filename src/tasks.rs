use crate::Grind;
use crate::shell;

pub fn execute_task(grind: Grind, task: String) {
    if task.to_lowercase() == "list" {
        println!("available tasks:\n");
        for k in grind.project.tasks.keys() {
            println!(" - {} ", k);
        }
        println!("");
        return;
    }

    let target = grind
        .project
        .tasks
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(&task));

    if let Some((k, v)) = target {
        println!("==> executing task [{}]", k);
        let out = shell(&v);
        println!("{}", out);
    } else {
        println!("unknown task '{}', available tasks:\n", task);
        for k in grind.project.tasks.keys() {
            println!(" - {} ", k);
        }
        println!("");
    }
}
