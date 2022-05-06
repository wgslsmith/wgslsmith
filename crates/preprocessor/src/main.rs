use std::io::Read;

fn main() {
    let input = read_stdin().unwrap();
    let processed = input.replace("@stage(compute)", "@compute");
    println!("{processed}");
}

fn read_stdin() -> std::io::Result<String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
