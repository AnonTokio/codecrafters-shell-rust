use std::io::Write;

use crate::{
    builtin::ExitCode,
    command::{Execute, Parse, ParseCommandResult},
    redirect::{Reader, Writer},
};

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

    /// For each NAME, specify how arguments are to be completed.
    names: Vec<String>,
}

impl Parse for CompleteCommand {
    fn parse(_command: &str, args: &[String]) -> ParseCommandResult<Self>
    where
        Self: std::marker::Sized,
    {
        let mut print = false;
        let mut remove = false;
        let mut names = vec![];
        for arg in args {
            match arg.as_str() {
                "-p" => print = true,
                "-r" => remove = true,
                name => names.push(name.to_string()),
            }
        }
        Ok(CompleteCommand {
            print,
            remove,
            names,
        })
    }
}

impl Execute for CompleteCommand {
    fn execute(
        &self,
        _reader: Reader,
        _output_writer: Writer,
        mut error_writer: Writer,
        _background: bool,
    ) -> ExitCode {
        if self.print {
            for name in self.names.iter() {
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

        0
    }
}
