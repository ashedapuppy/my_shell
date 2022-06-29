use anyhow::{Result, Context};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio}, env,
};

#[derive(Default)]
pub struct ShellCommand {
    pub name: String,
    pub arguments: Vec<String>,
}

impl ShellCommand {
    /// Taking a string and splitting it into a command and its arguments.
    pub fn new(name: String) -> Self {
        let mut parts = name.trim().split_whitespace();
        let command = parts.next().unwrap().to_string();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        Self {
            name: command,
            arguments: args,
        }
    }
}

/// It takes a `ShellCommand` and a `PathBuf` and
/// returns a `ControlFlow<()>`, to tell main whether to continue or break the loop
///
/// Arguments:
///
/// * `command`: The ShellCommand struct that contains the command and arguments.
/// * `path`: The current working directory.
/// * `previous_command`: This is the last command that was run. If the user presses the up arrow, we
/// want to run the same command again.
///
/// Returns:
///
/// A ControlFlow<()>
pub fn cd(
    command: &ShellCommand,
    previous_command: &mut Option<Child>,
) -> Result<PathBuf> {
    // This is a way to get the first argument of the command. If there are no arguments, it will
    // return "/".
    let new_dir = command
        .arguments
        .iter()
        .peekable()
        .peek()
        .map_or("/", |x| *x);
    // Creating a new path from the new directory.
    let new_path = fs::canonicalize(PathBuf::from(new_dir)).context("cd: failed to find directory")?;
    // Setting the current directory to the new path.
    env::set_current_dir(&new_path).context("failed to set current dir variable")?;
    *previous_command = None;
    Ok(new_path)
}

/// It takes a previous command, an iterator over the remaining commands, and the current command, and
/// returns a new command
///
/// Arguments:
///
/// * `previous_command`: The previous command in the pipeline.
/// * `cmd_iter`: This is a mutable iterator over the commands that are left to execute.
/// * `command`: The command to execute
///
/// Returns:
///
/// A child process
pub fn execute(
    previous_command: Option<Child>,
    cmd_iter: &mut std::iter::Peekable<std::slice::Iter<ShellCommand>>,
    command: &ShellCommand,
) -> Option<Child> {
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
        Ok(output) => Some(output),
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}
