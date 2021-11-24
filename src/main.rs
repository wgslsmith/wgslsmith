use clap::Parser;
use wgslsmith::generator::Generator;
use wgslsmith::types::TypeConstraints;

#[derive(Parser)]
struct Options {
    #[clap(short, long, about = "Print ast instead of WGSL code")]
    debug: bool,
}

fn main() {
    let options = Options::parse();
    let expr = Generator::new().gen_expr(TypeConstraints::INT);

    if options.debug {
        println!("{:#?}", expr);
    } else {
        println!("{}", expr);
    }
}
