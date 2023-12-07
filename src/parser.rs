use std::fmt;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};


pub struct FinishedShushCmd<'a> {
    shush_cmd: &'a ShushCmd,
    exit_code: bool,
    timestamp: u128,
}

impl FinishedShushCmd<'_> {
    pub fn append_to_histfile(self) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .open("/Users/victor/.shush_hist")?;

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

#[derive(Debug)]
enum BuiltInCommands {
    CD,
}

#[derive(Debug)]
pub struct ShushCmd {
    program: String,
    arguments: Vec<String>,
    built_in: Option<BuiltInCommands>,
}

impl ShushCmd {
    pub fn parse_command(line: String) -> Option<Self> {
        let words: Vec<String> = line
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();
        let program = words.first()?.to_string();
        let built_in : Option<BuiltInCommands> = match program.as_str() {
            "cd" => Some(BuiltInCommands::CD),
            _ => None,
        };
        Some(Self {
            program,
            arguments: words[1..].to_vec(),
            built_in,
        })
    }

    pub fn execute_command(&self) -> anyhow::Result<FinishedShushCmd> {
        match self.built_in {
            Some(BuiltInCommands::CD) => self.change_dir(),
            None => self.execute_program(),
        }
    }

    fn execute_program(&self) -> anyhow::Result<FinishedShushCmd> {
        let mut cmd_result = Command::new(&self.program).args(&self.arguments).spawn()?;
        let exit_status = cmd_result.wait()?;
        Ok(FinishedShushCmd {
            shush_cmd: self,
            exit_code: exit_status.success(),
            timestamp: get_epoch_ms(),
        })
    }

    fn change_dir(&self) -> anyhow::Result<FinishedShushCmd> {
        if self.arguments.len() != 1 {
            return Err(anyhow!("cd program only accepts one argument"));
        };
        match self.arguments.first() {
            Some(arg) => {
                let path = std::path::Path::new(arg);
                std::env::set_var("OLDPWD", std::env::current_dir()?);
                std::env::set_current_dir(path)?;
                std::env::set_var("PWD", path);
                Ok(FinishedShushCmd {
                    shush_cmd: self,
                    exit_code: true,
                    timestamp: get_epoch_ms(),
                })
            }
            None => return Err(anyhow!("no argument found")),
        }
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

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
