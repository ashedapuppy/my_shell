use std::ops::ControlFlow;
use std::path::PathBuf;
use std::env;

use clap::Parser;
use color_eyre::eyre::Result;
use rustyline::Editor;

mod readln;
mod cmd;

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

fn build_prompt(path: &PathBuf, prompt: &str) -> String {
    let new_path: PathBuf = path.clone();
    let mut full_prompt = new_path
        .into_os_string()
        .into_string()
        .unwrap();
    full_prompt.push_str(prompt);
    full_prompt
}

fn main() -> Result<()>{
    color_eyre::install()?;

    // parse command line arguments
    let args = Arguments::parse();

    // initialise the terminal input with rustyline completion
    let mut rl: Editor<readln::DIYHinter> = Editor::new();
    rl.set_helper(Some(readln::DIYHinter { hints: readln::diy_hints() }));

    // set the path variable used in the rest of the program
    let mut path = args.path.clone();
    env::set_current_dir(&path)?;

    shell_loop(&mut path, &args, &mut rl)?;
    Ok(())
}

fn shell_loop(path: &mut PathBuf, args: &Arguments, rl: &mut Editor<readln::DIYHinter>) -> Result<(), color_eyre::Report> {
    loop {
        let prompt = build_prompt(path, &args.prompt);
        let input = readln::input(rl, &prompt)?;
        let commands: Vec<cmd::ShellCommand> = input
            .trim()
            .split(" | ")
            .map(|s| cmd::ShellCommand::new(s.to_string()))
            .collect();
        let mut previous_command = None;
        let mut cmd_iter = commands.iter().peekable();
        while let Some(command) = cmd_iter.next() {
            match command.name.as_str() {
                "exit" => return Ok(()),

                "cd" => {
                    // break out of the loop if the cd command fails, preventing following commands
                    if let ControlFlow::Break(_) = cmd::cd(command, path, &mut previous_command) {
                        break;
                    }
                },

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

