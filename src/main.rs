use clap::Parser;
use wgslsmith::generator::Generator;

#[derive(Parser)]
struct Options {
    #[clap(short, long, about = "Print ast instead of WGSL code")]
    debug: bool,
}

fn main() {
    let options = Options::parse();
    let shader = Generator::new().gen_module();

    if options.debug {
        println!("{:#?}", shader);
    } else {
        println!("{}", shader);
    }
}
