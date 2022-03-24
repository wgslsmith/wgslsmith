use std::io::Read;

fn main() -> eyre::Result<()> {
    let input = read_stdin()?;
    let ast = parser::parse(&input);
    let result = reconditioner::recondition(ast);

    println!("{}", include_str!("prelude.wgsl"));

    if result.loop_count > 0 {
        println!(
            "var<private> LOOP_COUNTERS: array<u32, {}>;",
            result.loop_count
        );
        println!();
    }

    println!("{}", result.ast);

    Ok(())
}

fn read_stdin() -> eyre::Result<String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
