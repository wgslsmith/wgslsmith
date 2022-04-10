use std::error::Error;

use parser::DebugModule;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("missing argument: path to shader");

    let source = std::fs::read_to_string(path)?;
    let ast = parser::parse(&source);

    print!("{}", DebugModule(&ast));

    Ok(())
}
