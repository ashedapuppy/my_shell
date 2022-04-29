use std::{path::PathBuf, ops::ControlFlow, process::{Child, Stdio, Command}, env};

use crate::ShellCommand;

pub fn cd(command: &ShellCommand, path: &mut PathBuf, previous_command: &mut Option<Child>) -> ControlFlow<()> {
    let new_dir = command.arguments
        .iter()
        .peekable()
        .peek()
        .map_or("/", |x| *x);
    let new_path = PathBuf::from(new_dir);
    match env::set_current_dir(&new_path) {
        Err(_) => {
            eprintln!("could not open directory '{:?}'", path);
            return ControlFlow::Break(())
        }
        Ok(_) => *path = new_path
    };
    *previous_command = None;
    ControlFlow::Continue(())
}

pub fn execute(previous_command: Option<Child>, cmd_iter: &mut std::iter::Peekable<std::slice::Iter<ShellCommand>>, command: &ShellCommand) -> Option<Child> {
    let stdin = if let Some(child) = previous_command {
        Stdio::from(child.stdout.unwrap())
    } else {
        Stdio::inherit()
    };
    let stdout = if cmd_iter.peek().is_some() {
        Stdio::piped()
    } else {
        Stdio::inherit()
    };
    let output = Command::new(&command.name)
        .args(&command.arguments)
        .stdin(stdin)
        .stdout(stdout)
        .spawn();
    match output {
        Ok(output) => { 
            return Some(output)
        },
        Err(e) => {
            eprintln!("{}", e);
            return None
        },
    };
}