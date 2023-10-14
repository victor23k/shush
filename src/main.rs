use read_input::IO;

mod read_input;

fn main() {
    let mut io = IO::build_io();
    let line = io.read_from_stdin().unwrap();
    io.write_to_stdout(line.as_bytes()).unwrap();
}
