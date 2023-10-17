use parser::ShushCmd;
use read_input::IO;

mod parser;
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

        let cmd = ShushCmd::parse_command(line);
        let finished_cmd = cmd.execute_command().unwrap();
        finished_cmd.append_to_histfile().unwrap();
    }
}
