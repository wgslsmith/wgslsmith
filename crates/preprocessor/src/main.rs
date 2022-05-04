use std::io::Read;

fn main() {
    let options = preprocessor::Options {
        concise_stage_attrs: true,
        module_scope_constants: false,
    };

    let processed = preprocessor::preprocess(options, &read_stdin().unwrap());

    println!("{processed}");
}

fn read_stdin() -> std::io::Result<String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
