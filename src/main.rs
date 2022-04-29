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

#[derive(Default)]
pub struct ShellCommand {
    name: String,
    arguments: Vec<String>,
}

impl ShellCommand {
    fn new(name: String) -> Self { 
        let mut parts = name.trim().split_whitespace();
        let command = parts.next().unwrap().to_string();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        Self { 
            name: command,
            arguments: args,
        } 
    }
}



fn main() -> Result<()>{
    color_eyre::install()?;

    let mut rl: Editor<readln::DIYHinter> = Editor::new();
    rl.set_helper(Some(readln::DIYHinter { hints: readln::diy_hints() }));

    let args = Arguments::parse();
    let mut path = args.path;
    env::set_current_dir(&path)?;

    loop {
        let prompt = build_prompt(&path, &args.prompt);
        let input = readln::input(&mut rl, &prompt)?;
        let commands: Vec<ShellCommand> = input
            .trim()
            .split(" | ")
            .map(|s| ShellCommand::new(s.to_string()))
            .collect();
        let mut previous_command = None;
        let mut cmd_iter = commands.iter().peekable();
        while let Some(command) = cmd_iter.next() {
            match command.name.as_str() {
                "exit" => return Ok(()),

                "cd" => {
                    if let ControlFlow::Break(_) = cmd::cd(command, &mut path, &mut previous_command) {
                        continue;
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

