use crate::builtin::BuiltInCommands;
use crate::timestamps;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::process::Command;

/// A `FinishedShushCmd` has a lifetime that begins when a `ShushCmd` finishes execution and ends
/// when it is serialized to the history file.
pub struct FinishedShushCmd<'a> {
    shush_cmd: &'a ShushCmd,
    exit_code: bool,
    timestamp: u128,
}

impl FinishedShushCmd<'_> {
    pub fn new<'a>(cmd: &'a ShushCmd, exit_code: bool, timestamp: u128) -> FinishedShushCmd<'a> {
        FinishedShushCmd {
            shush_cmd: cmd,
            exit_code,
            timestamp,
        }
    }

    pub fn append_to_histfile(self) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .open("/Users/victor/.shush_hist")?; // TODO: load from config and set default

        writeln!(file, "{}", self)?;
        Ok(())
    }
}

impl fmt::Display for FinishedShushCmd<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{};{};{}",
            &self.timestamp.to_string(),
            &self.shush_cmd.to_string(),
            &self.exit_code.to_string(),
        )
    }
}

/// Executable command with arguments. It can be any of the builtin commands or an external
/// program.
///
/// It is used to parse input into a `ShushCmd`, execute it, and create a `FinishedShushCmd` struct
/// to hold its finish status.
///
/// ## Piping commands
///
/// TODO: implement
#[derive(Debug)]
pub struct ShushCmd {
    program: String,
    arguments: Vec<String>,
    builtin: Option<BuiltInCommands>,
    output: Option<File>,
}

impl ShushCmd {
    pub fn parse_command(line: String) -> Option<Self> {
        let output_path = line.split(">").last();
        let output = match output_path {
            Some(path) => File::options()
                .append(true)
                .create(true)
                .write(true)
                .open(path)
                .ok(),
            None => None,
        };
        let words: Vec<String> = line
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();
        let program = words.first()?.to_string();
        let builtin: Option<BuiltInCommands> = match program.as_str() {
            "cd" => Some(BuiltInCommands::CD),
            _ => None,
        };
        let arguments = words[1..]
            .to_vec()
            .iter()
            .map(|arg| -> String {
                if arg.starts_with("$") {
                    std::env::var(arg.split("$").last().unwrap_or("")).unwrap_or(arg.to_string())
                } else {
                    arg.to_string()
                }
            })
            .collect();
        Some(Self {
            program,
            arguments,
            builtin,
            output,
        })
    }

    pub fn execute_command(&self) -> anyhow::Result<FinishedShushCmd> {
        match &self.builtin {
            Some(builtin) => builtin.run(self),
            None => self.execute_program(),
        }
    }

    pub fn n_args(&self) -> usize {
        self.arguments.len()
    }

    pub fn args(&self) -> Vec<String> {
        self.arguments.clone()
    }

    fn execute_program(&self) -> anyhow::Result<FinishedShushCmd> {
        let mut cmd_result = Command::new(&self.program).args(&self.arguments).spawn()?;
        let exit_status = cmd_result.wait()?;
        Ok(FinishedShushCmd {
            shush_cmd: self,
            exit_code: exit_status.success(),
            timestamp: timestamps::get(),
        })
    }

}

impl fmt::Display for ShushCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format the program and arguments as a command
        write!(f, "{}", &self.program)?;

        for arg in &self.arguments {
            write!(f, " {}", arg)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FinishedShushCmd;
    use super::ShushCmd;

    #[test]
    fn executes_builtin_command() {
        let input = "cd /";
        let cmd = ShushCmd::parse_command(input.to_string()).unwrap();
        let finished_cmd = cmd.execute_command().unwrap();

        assert!(finished_cmd.exit_code == true);
    }

    #[test]
    fn executes_program() {
        let input = "echo e";
        let cmd = ShushCmd::parse_command(input.to_string()).unwrap();
        let finished_cmd = cmd.execute_command().unwrap();

        assert!(finished_cmd.exit_code == true);
    }
}
