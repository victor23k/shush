use crate::gap_buffer::GapBuffer;
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

#[derive(Debug)]
pub struct Editor {
    buffer: GapBuffer,
    cursor: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::new(),
            cursor: 0,
        }
    }

    pub fn move_cursor_left(&mut self) -> Option<&str> {
        if self.cursor > 0 {
            self.cursor -= 1;
            Some("\x1B[D")
        } else {
            None
        }
    }

    pub fn move_cursor_right(&mut self) -> Option<&str> {
        if self.cursor < self.buffer.text_len() {
            self.cursor += 1;
            Some("\x1B[C")
        } else {
            None
        }
    }

    pub fn move_cursor_to_start(&mut self) -> &str {
        self.cursor = 0;
        "\x1B[5G"
    }

    pub fn delete_backwards(&mut self, io: &mut IO) {
        if self.cursor == 0 {
            return
        }
        self.buffer.move_gap_to_cursor(self.cursor);
        self.buffer.delete_backwards(1);
        self.cursor -= 1;
        self.clear_line(io);
        self.write_line(io);
    }

    pub fn new_line(&mut self, io: &mut IO) {
        io.write_to_stdout("\n🤫> ".as_bytes());
        io.write_to_stdout(self.move_cursor_to_start().as_bytes());
    }

    pub fn clean_buffer(&mut self) {
        self.buffer.clear_buffer_text();
    }

    fn write_line(&mut self, io: &mut IO) {
        io.write_to_stdout("🤫> ".as_bytes());
        io.write_to_stdout(self.get_buffer_text().expect("buffer text wrong").as_bytes());
        io.write_to_stdout(format!("\x1b[{}G", self.cursor + 5).as_bytes()); // move cursor
        io.write_to_stdout(format!("\x1b[?25h").as_bytes()); // show cursor
        // io.write_to_stderr(format!("{:?}\n", self.buffer).as_bytes());
        // io.write_to_stderr(format!("{:?}", self.cursor).as_bytes());
    } 

    fn clear_line(&self, io: &mut IO) {
        io.write_to_stdout(format!("\x1b[?25l").as_bytes()); // hide cursor
        io.write_to_stdout("\x1B[2K\r".as_bytes());
    }

    /// In this function we have to re-render the line. Send ANSI code for clear line, then
    /// show the text stored in the buffer. The cursor stays in place.
    pub fn write_to_buffer(&mut self, slice: &[u8], io: &mut IO) {
        self.clear_line(io);
        self.buffer.move_gap_to_cursor(self.cursor);
        self.buffer.insert(slice);
        self.cursor += slice.len();
        self.write_line(io);
    }

    pub fn get_buffer_text(&self) -> Result<String, std::string::FromUtf8Error> {
        let string = self.buffer.get_text()?;
        Ok(string)
    }
}
