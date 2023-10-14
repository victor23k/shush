use read_input::IO;

mod read_input;

fn main() {
    let mut io = IO::build_io();
    loop {
        io.write_to_stdout("🤫> ".as_bytes()).unwrap();
        io.flush_stdout().unwrap();

        let line = io.read_from_stdin().unwrap();

        if line == "exit\n" || line == "shush\n" {
            io.write_to_stdout("shushing...".as_bytes()).unwrap();
            break;
        }

        io.write_to_stdout(line.as_bytes()).unwrap();
    }
}
