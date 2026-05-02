use std::{collections::HashSet, env, path::PathBuf, process, sync::RwLock};

use is_executable::IsExecutable;
use lazy_static::lazy_static;

use crate::{
    backgrond::BACKGROUDN_MANAGER,
    builtin::ExitCode,
    command::{Args, Execute, Parse, ParseCommandError, ParseCommandResult},
    redirect::{Reader, Writer},
};

lazy_static! {
    pub static ref PATH_ENV: RwLock<String> = RwLock::new(load_env_path());
    pub static ref PATHS: RwLock<HashSet<PathBuf>> = RwLock::new(HashSet::from_iter(load_paths()));
}

pub fn load_env_path() -> String {
    env::var("PATH").expect("Invalid $PATH")
}

#[allow(unused)]
pub fn load_paths() -> Vec<PathBuf> {
    //TODO 是否可以用 HashSet，还是应该用 Vec?
    env::split_paths(&load_env_path())
        .filter(|path| path.is_dir())
        .collect()
}

pub fn find_in_path(executable: &str) -> Option<PathBuf> {
    let executable = PathBuf::from(executable);
    if executable.exists() && executable.is_executable() {
        return Some(executable);
    }

    for dir in env::split_paths(&load_env_path()) {
        let candidate = dir.join(&executable);
        if candidate.exists() && candidate.is_executable() {
            return Some(candidate);
        }
    }
    None
}

#[derive(Debug, PartialEq, Eq)]
pub struct Executable {
    pub name: String,
    pub path: PathBuf,
    pub args: Args,
}

impl Executable {
    pub fn new(name: String, path: PathBuf, args: Args) -> Self {
        Self { name, path, args }
    }
}

impl Parse for Executable {
    fn parse(command: &str, args: &[String]) -> ParseCommandResult<Self>
    where
        Self: std::marker::Sized,
    {
        if let Some(exec_path) = find_in_path(command) {
            Ok(Executable::new(
                command.to_string(),
                exec_path,
                args.to_vec(),
            ))
        } else {
            Err(ParseCommandError::ExecutableNotExists(command.to_string()))
        }
    }
}

impl Execute for Executable {
    fn execute(
        &self,
        reader: Reader,
        output_writer: Writer,
        error_writer: Writer,
        background: bool,
    ) -> ExitCode {
        if let Ok(mut job) = process::Command::new(&self.name)
            .args(&self.args)
            .stdin(reader)
            .stdout(output_writer)
            .stderr(error_writer)
            .spawn()
        {
            if !background {
                if let Ok(exit_status) = job.wait() {
                    return exit_status.code().unwrap_or(-1);
                }
            } else if let Ok(mut bg_manager) = BACKGROUDN_MANAGER.lock() {
                let command = format!("{} {}", self.name, self.args.join(" "));
                bg_manager.add_job(command, job);
                return 0;
            }
        }
        -1
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::set_env_path;

    use super::*;

    #[test]
    fn test_find_in_path() {
        set_env_path();
        assert_eq!(find_in_path("ls"), Some(PathBuf::from("/usr/bin/ls")));
    }
}
