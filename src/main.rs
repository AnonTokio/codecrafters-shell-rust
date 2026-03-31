use std::sync::Mutex;

use lazy_static::lazy_static;
use rustyline::{CompletionType, Config, EditMode, Editor, history::FileHistory};

use crate::{
    helper::ShellHelper,
    history::{CURRENT_SESSION_HISTORY, load_history, save_history},
    parser::parse_tokens,
    tokenize::tokenize,
};

mod backgrond;
mod builtin;
mod command;
mod completer;
mod executable;
mod helper;
mod history;
mod parser;
mod redirect;
mod tokenize;
#[macro_use]
mod utils;
mod validator;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

static PROMPT: &str = "$ ";
lazy_static! {
    static ref HISTORY_FILE: String = std::env::var("HISTFILE").unwrap_or(".history".to_string());
}

lazy_static! {
    pub static ref RL: Mutex<Editor<ShellHelper, FileHistory>> = {
        let helper = ShellHelper::new();
        let config = Config::builder()
            .history_ignore_space(true)
            .auto_add_history(true)
            .edit_mode(EditMode::Emacs)
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config).expect("Failed to build Editor");
        rl.set_helper(Some(helper));
        Mutex::new(rl)
    };
}

fn main() {
    utils::config_logger();

    load_history(HISTORY_FILE.as_str()).ok();

    loop {
        let line = RL.lock().unwrap().readline(PROMPT);
        match line {
            Ok(line) => {
                CURRENT_SESSION_HISTORY
                    .lock()
                    .expect("Failed to get current session history")
                    .push(line.clone());

                let tokens = tokenize(&line);
                match parse_tokens(&tokens) {
                    Ok(command_exec_vec) => {
                        for command_exec in command_exec_vec {
                            command_exec.execute();
                        }
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }

    save_history(HISTORY_FILE.as_str(), true).ok();
}
