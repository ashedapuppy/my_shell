use std::path::PathBuf;
use std::process::{Command, Stdio, Child};
use std::env;

use clap::Parser;
use color_eyre::eyre::Result;
use rustyline::Editor;

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

    let mut rl: Editor<readln::DIYHinter> = Editor::new();
    rl.set_helper(Some(readln::DIYHinter { hints: readln::diy_hints() }));

    let args = Arguments::parse();
    let mut path = args.path;
    env::set_current_dir(&path)?;

    loop {
        let prompt = build_prompt(&path, &args.prompt);
        let input = readln::input(&mut rl, &prompt)?;
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "exit" => return Ok(()),

                "cd" => {
                    let new_dir = args
                        .peekable()
                        .peek()
                        .map_or("/", |x| *x);
                    let new_path = PathBuf::from(new_dir);
                    match env::set_current_dir(&new_path) {
                        Err(_) => {
                            eprintln!("could not open directory '{:?}'", path);
                            continue
                        }
                        Ok(_) => path = new_path
                    };
                    previous_command = None;
                },

                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => { 
                            previous_command = Some(output); 
                        },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait()?;
        }
        println!();
    }
}