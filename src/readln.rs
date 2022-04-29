// for hints
use std::{collections::HashSet, process};
use rustyline::{hint::{Hint, Hinter}, Context, Editor, error::ReadlineError};
use rustyline_derive::{Completer, Helper, Highlighter};

// for validator
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Result;

#[derive(Completer, Helper, Highlighter)]
pub struct DIYHinter {
    // It's simple example of rustyline, for more efficient, please use ** radix trie **
    pub hints: HashSet<CommandHint>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                // expect hint after word complete, like redis cli, add condition:
                // line.ends_with(" ")
                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

// TODO: validate user input
impl Validator for DIYHinter {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        #[allow(unused_imports)]
        use ValidationResult::{Incomplete, Invalid, Valid};

        #[allow(unused_variables)]
        let input = ctx.input();
        // let result = if !input.starts_with("SELECT") {
        //     Invalid(Some(" --< Expect: SELECT stmt".to_owned()))
        // } else if !input.ends_with(';') {
        //     Incomplete
        // } else {
        //     Valid(None)
        // };
        Ok(Valid(None))
    }
}

// TODO: add more completion support
pub fn diy_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();
    set.insert(CommandHint::new("exit", "exit"));
    set
}

pub fn input(rl: &mut Editor<DIYHinter>, prompt: &str) -> Result<String> {
    match rl.readline(prompt) {
        Ok(line) => {
            rl.add_history_entry(line.as_str());
            Ok(line)
        }
        Err(ReadlineError::Interrupted) => {
            println!("CTRL-C");
            process::exit(0);
        }
        Err(ReadlineError::Eof) => {
            println!("CTRL-D");
            process::exit(0);
        }
        Err(err) => {
            println!("Error: {:?}", err);
            Err(err.into())
        }
    }
}