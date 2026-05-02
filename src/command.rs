use std::io::Write;

use thiserror::Error;

use crate::{
    builtin::{BUILTIN_COMMANDS, BuiltinCommand, ExitCode},
    executable::Executable,
    redirect::{Reader, Writer},
};

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParseCommandError {
    #[error("{}: not enough arguments", .0)]
    LessArgs(String, Args, usize),

    #[error("{}: too many arguments", .0)]
    MoreArgs(String, Args, usize),

    #[error("Unknown parameter: {}", .0)]
    UnknownParam(String),

    #[error( "File {} does not exists or is not a file.", .0)]
    FileNotExists(String),

    #[error( "Executable {} does not exists or is not a file.", .0)]
    ExecutableNotExists(String),

    #[error("Failed to parse integer: {}", .0)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("The error type for errors that can never happen: {}", .0)]
    Infallible(#[from] std::convert::Infallible),
}

pub type ParseCommandResult<T> = std::result::Result<T, ParseCommandError>;

pub trait Execute {
    fn execute(
        &self,
        reader: Reader,
        output_writer: Writer,
        error_writer: Writer,
        background: bool,
    ) -> ExitCode;
}

pub trait Parse {
    fn parse(command: &str, args: &[String]) -> ParseCommandResult<Self>
    where
        Self: std::marker::Sized;
}

pub type Args = Vec<String>;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Empty,
    BuiltinCommand(BuiltinCommand),
    Executable(Executable),
    Unknown(UnknownCommand),
}

impl Parse for Command {
    fn parse(command: &str, args: &[String]) -> ParseCommandResult<Self>
    where
        Self: std::marker::Sized,
    {
        let command = if command.is_empty() {
            Command::Empty
        } else if BUILTIN_COMMANDS.contains(command) {
            Command::BuiltinCommand(BuiltinCommand::parse(command, args)?)
        } else if let Ok(exec) = Executable::parse(command, args) {
            Command::Executable(exec)
        } else {
            Command::Unknown(UnknownCommand::new(command.to_string(), args.to_vec()))
        };
        Ok(command)
    }
}

impl Execute for Command {
    fn execute(
        &self,
        reader: Reader,
        output_writer: Writer,
        mut error_writer: Writer,
        background: bool,
    ) -> ExitCode {
        match self {
            Command::Empty => 0,
            Command::BuiltinCommand(builtin_command) => {
                builtin_command.execute(reader, output_writer, error_writer, background)
            }
            Command::Executable(exec) => {
                exec.execute(reader, output_writer, error_writer, background)
            }
            Command::Unknown(unknown) => {
                let _ = writeln!(error_writer, "{}: command not found", unknown.command);
                0
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownCommand {
    pub command: String,
    pub args: Args,
}

impl UnknownCommand {
    pub fn new(command: String, args: Args) -> Self {
        Self { command, args }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(matches!(Command::parse("", &[]), Ok(Command::Empty)));
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(
            Command::parse(
                "invalid_command",
                &["invalid".to_string(), "args".to_string()]
            )
            .unwrap(),
            Command::Unknown(UnknownCommand::new(
                "invalid_command".to_string(),
                vec!["invalid".to_string(), "args".to_string()]
            ))
        );
    }

    #[test]
    fn test_parse_exit() {
        assert_eq!(
            Command::parse("exit", &[]).unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(0))
        );
        assert_eq!(
            Command::parse("exit", &["123".to_string()]).unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(123))
        );
    }
}
