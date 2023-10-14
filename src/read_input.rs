// the first iteration of the shell will be reading what is passed to the program and printing it
use std::io::{self, Write};

pub struct IO {
    stdin: io::Stdin,
    stdout: io::Stdout,
    stderr: io::Stderr,
}

impl IO {
    pub fn read_from_stdin(&self) -> io::Result<String> {
        let mut buffer = String::new();
        self.stdin.read_line(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_to_stdout(&mut self, output: &[u8]) -> io::Result<()> {
        self.stdout.write_all(output)?;
        Ok(())
    }
    pub fn build_io() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }
}
