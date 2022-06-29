use std::env;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Result, Context};
use clap::Parser;
use rustyline::Editor;

mod cmd;
mod readln;

// rust shell implementation
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(long, default_value = "/", env = "PWD")]
    /// what path to start the shell at
    path: PathBuf,

    #[clap(short, long, default_value = " >> ", env = "RSHELL_PROMPT")]
    /// prompt to display after path
    prompt: String,
}

/// It takes a path and a prompt, and returns a string that is the path and the prompt concatenated
/// together
///
/// Arguments:
///
/// * `path`: The current working directory.
/// * `prompt`: The prompt to display to the user.
///
/// Returns:
///
/// A String
fn build_prompt(path: &Path, prompt: &str) -> String {
    let new_path: PathBuf = path.to_path_buf();
    let mut full_prompt = new_path.into_os_string().into_string().unwrap();
    full_prompt.push_str(prompt);
    full_prompt
}

/// It reads a line of input, splits it into commands, and executes each command
///
/// Arguments:
///
/// * `path`: A mutable reference to a PathBuf struct. This is the current working directory.
/// * `args`: The arguments passed to the program.
/// * `rl`: &mut Editor<readln::DIYHinter>
///
/// Returns:
///
/// A Result<()>
fn shell_loop(
    args: &Arguments,
    rl: &mut Editor<readln::DIYHinter>,
) -> Result<()> {
    loop {
        let path = env::current_dir()?;
        let prompt = build_prompt(&path, &args.prompt);
        let input = readln::input(rl, &prompt).context("failed to parse input")?;
        // Creating a vector of ShellCommand structs from the input string.
        let commands: Vec<cmd::ShellCommand> = input
            .trim()
            .split("|")
            .map(|s| cmd::ShellCommand::new(s.to_string()))
            .collect();
        let mut previous_command = None;
        let mut cmd_iter = commands.iter().peekable();
        while let Some(command) = cmd_iter.next() {
            match command.name.as_str() {
                "exit" => return Ok(()),

                "cd" => {
                    // break out of the loop if the cd command fails, preventing following commands
                    if let Err(e) = cmd::cd(command, &mut previous_command) {
                        eprintln!("{e}");
                        break;
                    }
                }

                _ => {
                    previous_command = cmd::execute(previous_command, &mut cmd_iter, command);
                }
            }
        }
        if let Some(mut final_command) = previous_command {
            final_command.wait()?;
        }
        println!();
    }
}

fn main() -> Result<()> {
    // parse command line arguments
    let args = Arguments::parse();

    // initialise the terminal input with rustyline completion + validation
    // (both to be completed)
    let mut rl: Editor<readln::DIYHinter> = Editor::new();
    rl.set_helper(Some(readln::DIYHinter {
        hints: readln::diy_hints(),
    }));

    // set the path variable used in the rest of the program
    env::set_current_dir(&args.path).context("failed to initialise current dir")?;

    shell_loop(&args, &mut rl)?;
    Ok(())
}
