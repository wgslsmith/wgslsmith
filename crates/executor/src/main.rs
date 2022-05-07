use std::io::Read;

fn main() {
    let server = std::env::args().nth(1).unwrap();
    let metadata = std::env::args().nth(2).unwrap();
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    std::process::exit(executor::exec_shader(&server, &input, &metadata));
}
