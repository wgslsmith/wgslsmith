use std::io::Read;

fn main() -> eyre::Result<()> {
    let input = read_stdin()?;
    let ast = parser::parse(&input);
    println!("{}", ast);
    Ok(())
}

fn read_stdin() -> eyre::Result<String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
