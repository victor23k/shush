use std::fmt;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FinishedShushCmd<'a> {
    shush_cmd: &'a ShushCmd,
    status: ExitStatus,
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
            &self.status.success().to_string(),
        )
    }
}

#[derive(Debug)]
pub struct ShushCmd {
    program: String,
    arguments: Vec<String>,
}

impl ShushCmd {
    pub fn parse_command(line: String) -> Option<Self> {
        let words: Vec<String> = line
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();
        Some(Self {
            program: words.first()?.to_string(),
            arguments: words[1..].to_vec(),
        })
    }

    pub fn execute_command(&self) -> io::Result<FinishedShushCmd> {
        let mut cmd_result = Command::new(&self.program).args(&self.arguments).spawn()?;
        let exit_status = cmd_result.wait()?;
        Ok(FinishedShushCmd {
            shush_cmd: self,
            status: exit_status,
            timestamp: get_epoch_ms(),
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

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
