use std::{collections::HashMap, io::Write, sync::Mutex};

use lazy_static::lazy_static;

use crate::{
    builtin::ExitCode,
    command::{Execute, Parse, ParseCommandError, ParseCommandResult},
    redirect::{Reader, Writer},
};

lazy_static! {
    static ref COMPLETER_SCRIPT_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// complete: complete [-abcdefgjksuv] [-pr] [-DEI] [-o option] [-A action] [-G globpat] [-W wordlist] [-F function] [-C command] [-X filterpat] [-P prefix] [-S suffix] [name ...]
/// Specify how arguments are to be completed by Readline.
///
/// For each NAME, specify how arguments are to be completed.  If no options
/// are supplied, existing completion specifications are printed in a way that
/// allows them to be reused as input.
///
/// Options:
///   -p        print existing completion specifications in a reusable format
///   -r        remove a completion specification for each NAME, or, if no
///             NAMEs are supplied, all completion specifications
///   -D        apply the completions and actions as the default for commands
///             without any specific completion defined
///   -E        apply the completions and actions to "empty" commands --
///             completion attempted on a blank line
///   -I        apply the completions and actions to the initial (usually the
///             command) word
///
/// When completion is attempted, the actions are applied in the order the
/// uppercase-letter options are listed above. If multiple options are supplied,
/// the -D option takes precedence over -E, and both take precedence over -I.
///
/// Exit Status:
/// Returns success unless an invalid option is supplied or an error occurs.
#[derive(Debug, PartialEq, Eq)]
pub struct CompleteCommand {
    /// -p, print existing completion specifications in a reusable format
    print: bool,

    /// -r, remove a completion specification for each NAME, or, if no NAMEs are supplied, all completion specifications
    remove: bool,

    /// -C, register a completer script for a command
    completer_script: Option<String>,

    /// For each NAME, specify how arguments are to be completed.
    names: Vec<String>,
}

impl Parse for CompleteCommand {
    fn parse(command: &str, args: &[String]) -> ParseCommandResult<Self>
    where
        Self: std::marker::Sized,
    {
        let mut print = false;
        let mut remove = false;
        let mut completer_script = None;
        let mut names = vec![];

        let mut cnt = 0;
        let mut args_iter = args.iter();
        while let Some(arg) = args_iter.next() {
            cnt += 1;
            match arg.as_str() {
                "-p" => print = true,
                "-r" => remove = true,
                "-C" => {
                    cnt += 1;
                    let Some(script) = args_iter.next() else {
                        return Err(ParseCommandError::LessArgs(
                            command.to_string(),
                            args.to_vec(),
                            cnt,
                        ));
                    };
                    completer_script = Some(script.to_string());
                }
                name => names.push(name.to_string()),
            }
        }

        Ok(CompleteCommand {
            print,
            remove,
            completer_script,
            names,
        })
    }
}

impl Execute for CompleteCommand {
    fn execute(
        &self,
        _reader: Reader,
        mut output_writer: Writer,
        mut error_writer: Writer,
        _background: bool,
    ) -> ExitCode {
        let mut completer_script_map = COMPLETER_SCRIPT_MAP.lock().unwrap();
        if let Some(script) = &self.completer_script {
            for name in self.names.iter() {
                completer_script_map.insert(name.to_string(), script.to_string());
            }
        }

        if self.print {
            for name in self.names.iter() {
                if let Some(script) = completer_script_map.get(name) {
                    if writeln!(output_writer, "complete -C '{}' {}", script, name).is_err() {
                        return -1;
                    }
                } else {
                    if writeln!(
                        error_writer,
                        "complete: {}: no completion specification",
                        name
                    )
                    .is_err()
                    {
                        return -1;
                    }
                }
            }
        }

        0
    }
}
