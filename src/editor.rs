use crate::gap_buffer::GapBuffer;
use crate::read_input::IO;

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
