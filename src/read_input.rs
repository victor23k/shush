use std::io::Error;
use std::os::fd::{IntoRawFd, RawFd};
use std::{
    error,
    io::{self, Write},
};
use termios::*;

#[derive(Debug)]
pub struct IO {
    stdin_raw: RawFd,
    stdin: io::Stdin,
    stdout: io::Stdout,
    stderr: io::Stderr,
}

impl IO {
    /// Read byte to byte using libc::read()
    pub fn read_keypress(&mut self) -> Result<Vec<u8>, Error> {
        let mut vec = Vec::<u8>::new();
        loop {
            let mut buf = [0; 1];
            let res =
                unsafe { libc::read(self.stdin_raw, buf.as_mut_ptr() as *mut libc::c_void, 1) };
            if res == 0 {
                break;
            } else if res == 1 {
                vec.extend_from_slice(&buf);
                break;
            } else {
                let error = io::Error::last_os_error();
                return Err(error);
            }
        }
        Ok(vec)
    }

    pub fn read_from_stdin(&self) -> io::Result<String> {
        let buffer = String::new();
        Ok(buffer)
    }

    pub fn write_to_stdout(&mut self, output: &[u8]) -> io::Result<()> {
        let mut stdout = self.stdout.lock();
        stdout.write_all(output)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn write_to_stderr(&mut self, output: &[u8]) -> io::Result<()> {
        self.stderr.write_all(output)?;
        Ok(())
    }

    pub fn build_io() -> Result<Self, std::io::Error> {
        Ok(Self {
            stdin_raw: Self::tty_fd()?,
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
        })
    }

    pub fn flush_stdout(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    pub fn get_termios(&mut self) -> Result<Termios, Box<dyn error::Error>> {
        let og_termios = Termios::from_fd(self.stdin_raw)?;
        Ok(og_termios)
    }

    pub fn enable_raw_mode(&mut self, termios: &mut Termios) -> Result<(), Box<dyn error::Error>> {
        termios.c_lflag &= !(ICANON | ECHO);
        self.change_termios(termios)?;
        Ok(())
    }

    fn tty_fd() -> io::Result<RawFd> {
        let fd: RawFd = if unsafe { libc::isatty(libc::STDIN_FILENO) == 1 } {
            libc::STDIN_FILENO
        } else {
            print!("manual");
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")?
                .into_raw_fd()
        };

        Ok(fd)
    }

    pub fn change_termios(&mut self, termios: &Termios) -> Result<(), Box<dyn error::Error>> {
        Ok(tcsetattr(self.stdin_raw, TCSANOW, termios)?)
    }
}
