use clap::Parser;
use colored::*;
use color_eyre::eyre::Result;
use rustyline::Editor;


// program description
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
}

fn input(rl: &mut Editor<()>) -> Result<String> {
    let input_str = rl.readline(">> ")?;
    rl.add_history_entry(input_str.as_str());
    println!("User input: {}", input_str.red());
    Ok(input_str)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut rl = Editor::<()>::new();
    let _args = Arguments::parse();

    let _input = input(&mut rl)?;

    Ok(())
}