use std::error;
use std::io;

use evaluator::ShushCmd;
use read_input::{Editor, IO};

mod gap_buffer;
mod evaluator;
mod lexer;
mod read_input;
mod parser;
mod builtin;
mod timestamps;

fn main() {
    let mut io = IO::build_io().expect("Should able to build io");
    let og_termios = io
        .get_termios()
        .expect("Should be able to get termios struct");
    let mut termios = og_termios.clone();
    if let Err(error) = io.enable_raw_mode(&mut termios) {
        io.write_to_stderr(format!("Error while enabling raw mode: {:?}\n", error).as_bytes())
            .unwrap();
    };
    repl(&mut io);
    io.change_termios(&og_termios).unwrap();
}

enum SpecialKey {
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Home,
    End,
    Backspace,
    Delete,
    Enter,
    CtrlC,
    CtrlU,
    CtrlD,
    Quit,
    EOT,
    VTab,
}

fn repl(io: &mut IO) -> Result<(), Box<dyn error::Error>> {
    let mut editor = Editor::new();
    if let Err(error) = prompt(io) {
        io.write_to_stderr(format!("Error while showing the prompt: {:?}\n", error).as_bytes())
            .unwrap();
    }
    loop {
        let cmd: ShushCmd;
        let bytes = match io.read_keypress() {
            Ok(key) => key,
            Err(error) => {
                io.write_to_stderr(
                    format!("Error while reading line from stdin: {:?}\n", error).as_bytes(),
                )
                .unwrap();
                panic!();
            }
        };

        let key = match bytes.first() {
            Some(b'q') => Some(SpecialKey::Quit),
            Some(b'\n') => Some(SpecialKey::Enter),
            Some(4) => Some(SpecialKey::EOT),
            Some(11) => Some(SpecialKey::VTab),
            Some(126) => Some(SpecialKey::Home),
            Some(127) => Some(SpecialKey::Backspace),
            Some(32..=255) => {
                editor.write_to_buffer(bytes.as_slice(), io);
                None
            }
            // Escape sequence
            Some(b'\x1B') => match io.read_keypress()?.first() {
                Some(b'[') => {
                    let first_escape_char = io.read_keypress()?.first().copied();
                    match first_escape_char {
                        Some(0..=9) => match io.read_keypress()?.first() {
                            Some(b'~') => match first_escape_char {
                                Some(b'1') => Some(SpecialKey::Home),
                                Some(b'7') => Some(SpecialKey::Home),
                                _ => None,
                            },
                            _ => None,
                        },
                        Some(b'A') => Some(SpecialKey::UpArrow),
                        Some(b'B') => Some(SpecialKey::DownArrow),
                        Some(b'C') => Some(SpecialKey::RightArrow),
                        Some(b'D') => Some(SpecialKey::LeftArrow),
                        Some(b'H') => Some(SpecialKey::Home),
                        Some(b'F') => Some(SpecialKey::End),
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        };

        match key {
            Some(SpecialKey::Quit) | Some(SpecialKey::EOT) => {
                io.write_to_stdout("shushing...\n".as_bytes()).unwrap();
                break Ok(());
            }
            Some(SpecialKey::LeftArrow) => {
                if let Some(move_left) = editor.move_cursor_left() {
                    io.write_to_stdout(move_left.as_bytes())?;
                };
            }
            Some(SpecialKey::RightArrow) => {
                if let Some(move_right) = editor.move_cursor_right() {
                    io.write_to_stdout(move_right.as_bytes())?;
                };
            }
            Some(SpecialKey::Home) => {
                let move_to_start = editor.move_cursor_to_start();
                io.write_to_stdout(move_to_start.as_bytes())?;
            }
            Some(SpecialKey::End) => {
                let move_to_start = editor.move_cursor_to_start();
                io.write_to_stdout(move_to_start.as_bytes())?;
            }
            Some(SpecialKey::Backspace) => {
                editor.delete_backwards(io);
            }
            Some(SpecialKey::VTab) => {
                // autocomplete here, GL :)
            }
            Some(SpecialKey::Enter) => {
                io.write_to_stdout(b"\n");
                let cmd = match ShushCmd::parse_command(editor.get_buffer_text()?) {
                    Some(cmd) => cmd,
                    None => {
                        io.write_to_stderr("No command found\n".as_bytes()).unwrap();
                        continue;
                    }
                };
                let finished_cmd = match cmd.execute_command() {
                    Ok(finished_cmd) => finished_cmd,
                    Err(error) => {
                        io.write_to_stderr(format!("Command failed: {:?}\n", error).as_bytes())
                            .unwrap();
                        editor.clean_buffer();
                        editor.new_line(io);
                        continue;
                    }
                };
                if let Err(error) = finished_cmd.append_to_histfile() {
                    io.write_to_stderr(
                        format!("Error while saving command to histfile: {:?}\n", error).as_bytes(),
                    )
                    .unwrap();
                };
                editor.clean_buffer();
                editor.new_line(io);
            }
            None => (),
            _ => (),
        }
    }
}

fn prompt(io: &mut IO) -> io::Result<()> {
    io.write_to_stdout("🤫> ".as_bytes())?;
    Ok(())
}
